use once_cell::sync::Lazy;
use salvo_worker::WorkerService;
use std::sync::Arc;
use std::sync::Mutex;

use salvo_worker::salvo::{
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
mod timeout;

fn init_service() -> Arc<WorkerService> {
    let service: WorkerService = Arc::new(init_router()).into();
    let service = service
        .cors(init_cors())
        .hoop(salvo::logging::Logger::new())
        .catch_bad_request_and_not_found();
    Arc::new(service)
}

fn init_cors() -> CorsHandler {
    Cors::new()
        .allow_origin(["http://127.0.0.1:5800", "http://localhost:5800"])
        .allow_methods(vec![Method::GET, Method::POST, Method::DELETE])
        .allow_headers("authorization")
        .into_handler()
}

fn init_router() -> Router {
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

    #[allow(clippy::unwrap_used)]
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

    #[allow(clippy::unwrap_used)]
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
        .push(Router::with_path("/api-doc/openapi.json").get(open_api_handler))
        .get(rate_limiter::hello)
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
        .push(
            Router::with_path("open_api").push(
                Router::with_path("hello")
                    .get(open_api::hello)
                    .push(Router::with_path("{pet_id}").get(open_api::hello2)),
            ),
        )
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
                .hoop(size_limiter::max_size(100))
                .post(rate_limiter::hello),
        )
        // timeout
        .push(
            Router::with_path("timeout")
                .hoop(salvo::timeout::Timeout::new(std::time::Duration::from_secs(5)))
                .push(Router::with_path("slow").get(timeout::slow))
                .push(Router::with_path("fast").get(timeout::fast)),
        )
        // trailing slash
        .push(
            Router::with_path("trailing_slash").push(
                Router::with_hoop(trailing_slash::add_slash())
                    .push(Router::with_path("hello").get(rate_limiter::hello))
                    .push(Router::with_path("hello.world").get(rate_limiter::hello)),
            ),
        );

    // let doc = oapi::OpenApi::new("template api", "0.0.1").merge_router(&router);
    let mut router = router;
    for ui in salvo_worker::open_api::ui_all("/api-doc/openapi.json", None) {
        router = router.unshift(ui);
    }
    router
}

#[allow(unused)]
const OPEN_API_FILE: &str = "docs/open-api.json"; // 子项目根目录
const OPEN_API_CONTENT: &str = include_str!("../docs/open-api.json"); // 当前文件目录起始
#[handler]
async fn open_api_handler() -> &'static str {
    OPEN_API_CONTENT
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 更新 open-api.json
    #[ignore]
    #[test]
    fn update_open_api() {
        salvo_worker::open_api::update_open_api(init_router(), "Template Api Docs", "0.0.1", OPEN_API_FILE);
    }

    /// 使用 endpoint
    #[ignore]
    #[test]
    fn release_all_endpoints() {
        salvo_worker::open_api::release_all_endpoints("src");
    }

    /// 使用 handler
    #[ignore]
    #[test]
    fn release_all_handlers() {
        salvo_worker::open_api::release_all_handlers("src");
    }
}
