use once_cell::sync::Lazy;
use salvo_worker::WorkerService;
use salvo_worker::salvo::size_limiter::max_size;
use std::sync::Arc;
use std::sync::Mutex;

use salvo_worker::salvo::{
    self,
    cors::{Cors, CorsHandler},
    http::Method,
    *,
};

pub(crate) static WORKER_SERVICE: Lazy<Arc<WorkerService>> = Lazy::new(init_service);

mod affix_state;
mod basic_auth;
mod cache;
mod caching_headers;
mod catch_panic;
mod concurrency_limiter;
mod csrf;
mod flash_cookie;
mod flash_session;
mod logging;
mod open_api;
mod proxy;
mod rate_limiter;
mod request_id;
mod session;

fn init_service() -> Arc<WorkerService> {
    let service: WorkerService = init_router().into();
    let service = service.cors(init_cors());
    let service = service.hoop(salvo::logging::Logger::new());
    Arc::new(service)
}

fn init_cors() -> CorsHandler {
    let cors = Cors::new()
        .allow_origin(["http://127.0.0.1:5800", "http://localhost:5800"])
        .allow_methods(vec![Method::GET, Method::POST, Method::DELETE])
        .allow_headers("authorization")
        .into_handler();
    cors
}

fn init_router() -> Arc<Router> {
    let config = affix_state::Config {
        username: "root".to_string(),
        password: "pwd".to_string(),
    };

    let auth_handler = salvo_worker::salvo::basic_auth::BasicAuth::new(basic_auth::Validator);

    use salvo::cache::{Cache, RequestIssuer, WorkerStore};
    let short_cache = salvo::cache::Cache::new(
        WorkerStore::builder("KV_TEST".into())
            .time_to_live(std::time::Duration::from_secs(60))
            .build(),
        RequestIssuer::default(),
    );
    let long_cache = Cache::new(
        WorkerStore::builder("KV_TEST".into())
            .time_to_live(std::time::Duration::from_secs(120))
            .build(),
        RequestIssuer::default(),
    );

    let session_handler = salvo::session::SessionHandler::builder(
        salvo::session::CookieStore::new(),
        b"secretabsecretabsecretabsecretabsecretabsecretabsecretabsecretab", // cspell: disable-line
    )
    .build()
    .unwrap();

    // Configure CSRF token finder in form data
    let form_finder = salvo::csrf::FormFinder::new("csrf_token");

    // Initialize different CSRF protection methods
    // let bcrypt_csrf = bcrypt_cookie_csrf(form_finder.clone());
    let hmac_csrf = salvo::csrf::hmac_cookie_csrf(*b"01234567012345670123456701234567", form_finder.clone());
    let aes_gcm_cookie_csrf =
        salvo::csrf::aes_gcm_cookie_csrf(*b"01234567012345670123456701234567", form_finder.clone());
    let ccp_cookie_csrf = salvo::csrf::ccp_cookie_csrf(*b"01234567012345670123456701234567", form_finder.clone());

    let flash_session_handler = salvo::session::SessionHandler::builder(
        salvo::session::MemoryStore::new(),
        b"secretabsecretabsecretabsecretabsecretabsecretabsecretabsecretab", // cspell: disable-line
    )
    .build()
    .unwrap();

    use salvo::rate_limiter::{BasicQuota, FixedGuard, RateLimiter, RemoteIpIssuer};
    let limiter = RateLimiter::new(
        FixedGuard::new(),
        salvo::rate_limiter::WorkerStore::new("KV_TEST".into()),
        RemoteIpIssuer,
        BasicQuota::new(1, time::Duration::seconds(10)),
    );

    let router = Router::new()
        .get(hello)
        .push(Router::with_path("json").get(json))
        .push(Router::with_path("hello").get(json))
        // affix_state
        .push(
            Router::with_path("affix_state")
                .hoop(
                    salvo_worker::salvo::affix_state::inject(config)
                        // Inject a shared State instance into the request context
                        .inject(Arc::new(affix_state::State {
                            fails: Mutex::new(Vec::new()),
                        }))
                        // Insert custom data into the request context
                        .insert("custom_data", "I love this world!"),
                )
                // Register the hello handler for the root path
                .get(affix_state::hello)
                // Add an additional route for the path "/hello" with the same handler
                .push(Router::with_path("hello").get(affix_state::hello)),
        )
        // basic_auth
        .push(Router::with_path("basic_auth").push(Router::with_hoop(auth_handler).goal(basic_auth::hello)))
        // cache
        .push(
            Router::with_path("cache")
                .push(Router::with_path("short").hoop(short_cache).get(cache::short))
                .push(Router::with_path("long").hoop(long_cache).get(cache::long)),
        )
        // caching headers
        .push(
            Router::with_path("caching_headers").push(
                Router::with_hoop(salvo::caching_headers::CachingHeaders::new())
                    // .hoop(salvo::compression::Compression::new().min_length(0)) // compression not supported
                    .get(caching_headers::hello),
            ),
        )
        // catch panic
        .push(
            Router::with_path("catch_panic")
                .hoop(salvo::catch_panic::CatchPanic::new())
                .get(catch_panic::hello),
        )
        // catch panic
        .push(
            Router::with_path("concurrency_limiter")
                .hoop(salvo::concurrency_limiter::max_concurrency(1))
                .get(concurrency_limiter::index),
        )
        // session
        .push(
            Router::with_path("session")
                .hoop(session_handler)
                .get(session::home)
                .push(Router::with_path("login").get(session::login).post(session::login))
                .push(Router::with_path("logout").get(session::logout)),
        )
        // csrf
        .push(
            Router::with_path("csrf")
                .get(csrf::home)
                // // Bcrypt-based CSRF protection
                // .push(
                //     Router::with_hoop(bcrypt_csrf)
                //         .path("bcrypt")
                //         .get(csrf::get_page)
                //         .post(csrf::post_page),
                // )
                // HMAC-based CSRF protection
                .push(
                    Router::with_hoop(hmac_csrf)
                        .path("hmac")
                        .get(csrf::get_page)
                        .post(csrf::post_page),
                )
                // AES-GCM-based CSRF protection
                .push(
                    Router::with_hoop(aes_gcm_cookie_csrf)
                        .path("aes_gcm")
                        .get(csrf::get_page)
                        .post(csrf::post_page),
                )
                // ChaCha20Poly1305-based CSRF protection
                .push(Router::with_hoop(ccp_cookie_csrf).path("ccp").get(csrf::get_page)),
        )
        // flash cookie
        .push(
            Router::with_path("flash_cookie")
                .hoop(flash::CookieStore::new().into_handler())
                .push(Router::with_path("get").get(flash_cookie::get_flash))
                .push(Router::with_path("set").get(flash_cookie::set_flash)),
        )
        // flash session
        .push(
            Router::with_path("flash_session")
                .hoop(flash_session_handler)
                .hoop(flash::SessionStore::new().into_handler())
                .push(Router::with_path("get").get(flash_session::get_flash))
                .push(Router::with_path("set").get(flash_session::set_flash)),
        )
        // open api
        .push(Router::with_path("open_api").push(Router::with_path("hello").get(open_api::hello)))
        // proxy
        .push(
            Router::with_path("proxy")
                .push(Router::new().path("google/{**rest}").goal(salvo::proxy::Proxy::<
                    Vec<&str>,
                    salvo::proxy::ReqwestClient,
                >::new(
                    vec!["https://www.google.com"],
                    salvo::proxy::ReqwestClient::default(),
                )))
                .push(
                    Router::new()
                        .path("baidu/{**rest}")
                        .goal(salvo::proxy::Proxy::<Vec<&str>, _>::new(
                            vec!["https://www.baidu.com"],
                            salvo::proxy::ReqwestClient::default(),
                        )),
                ),
        )
        // rate limiter
        .push(Router::with_path("rate_limiter").hoop(limiter).get(rate_limiter::hello))
        // request id
        .push(
            Router::with_path("request_id")
                .hoop(RequestId::new())
                .get(request_id::hello),
        )
        // size limiter
        .push(
            Router::with_path("size_limiter")
                .hoop(max_size(100))
                .post(rate_limiter::hello),
        );

    // let doc = oapi::OpenApi::new("test api", "0.0.1").merge_router(&router);

    // let router = router
    //     .unshift(doc.into_router("/api-doc/openapi.json"))
    //     // .unshift(oapi::swagger_ui::SwaggerUi::new("/api-doc/openapi.json").into_router("/swagger-ui"))
    //     ;

    Arc::new(router)
}

#[handler]
async fn hello() -> Result<&'static str, ()> {
    Ok("Hello World")
}
#[handler]
async fn json(res: &mut Response) {
    #[derive(Serialize, Debug)]
    struct User {
        name: String,
    }
    res.render(Json(User { name: "jobs".into() }));
}
