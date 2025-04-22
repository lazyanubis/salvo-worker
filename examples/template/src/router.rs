use once_cell::sync::Lazy;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use salvo_worker::salvo::{self, *};

pub(crate) static ROUTER: Lazy<Arc<Router>> = Lazy::new(init_router);

mod affix_state;
mod basic_auth;
mod cache;
mod caching_headers;
mod catch_panic;

fn init_router() -> Arc<Router> {
    let config = affix_state::Config {
        username: "root".to_string(),
        password: "pwd".to_string(),
    };

    let auth_handler = salvo_worker::salvo::basic_auth::BasicAuth::new(basic_auth::Validator);

    use salvo::cache::{Cache, RequestIssuer, WorkerStore};
    let short_cache = salvo::cache::Cache::new(
        WorkerStore::builder("KV_TEST".into())
            .time_to_live(Duration::from_secs(60))
            .build(),
        RequestIssuer::default(),
    );
    let long_cache = Cache::new(
        WorkerStore::builder("KV_TEST".into())
            .time_to_live(Duration::from_secs(120))
            .build(),
        RequestIssuer::default(),
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
                    // .hoop(salvo::caching_headers::Co) // TODO compression
                    .get(caching_headers::hello),
            ),
        )
        // catch panic
        .push(
            Router::with_path("catch_panic")
                .hoop(salvo::catch_panic::CatchPanic::new())
                .get(catch_panic::hello),
        );
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
