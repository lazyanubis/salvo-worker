use once_cell::sync::Lazy;
use std::sync::Arc;

use salvo_worker::salvo::*;

pub(crate) static ROUTER: Lazy<Arc<Router>> = Lazy::new(init_router);

fn init_router() -> Arc<Router> {
    let router = Router::new()
        .get(hello)
        .push(Router::with_path("json").get(json))
        .push(Router::with_path("hello").get(json));
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
