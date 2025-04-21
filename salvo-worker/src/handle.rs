use worker::*;

/// handle request
pub async fn handle(req: Request, env: Env, ctx: Context) -> worker::Result<Response> {
    let router = Router::new();
    router
        .get_async("/hello", |_, _| async move { Response::ok("Hello, World!") })
        .run(req, env)
        .await
}
