use once_cell::sync::Lazy;
use std::sync::Arc;
use std::sync::Mutex;

use salvo_worker::salvo::*;

pub(crate) static ROUTER: Lazy<Arc<Router>> = Lazy::new(init_router);

mod affix_state;
mod basic_auth;

fn init_router() -> Arc<Router> {
    let config = affix_state::Config {
        username: "root".to_string(),
        password: "pwd".to_string(),
    };

    let auth_handler = salvo_worker::salvo::basic_auth::BasicAuth::new(basic_auth::Validator);

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
        .push(Router::with_path("basic_auth").push(Router::with_hoop(auth_handler).goal(basic_auth::hello)));
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
