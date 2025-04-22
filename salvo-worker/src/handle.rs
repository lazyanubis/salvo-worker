use futures::stream::TryStreamExt;
use std::sync::Arc;
use worker::*;

/// handle request
pub async fn handle(router: Arc<salvo_core::Router>, req: Request, env: Env, ctx: Context) -> worker::Result<Response> {
    // parse request
    let request: HttpRequest = req.try_into()?;
    let (parts, mut body) = request.into_parts();
    let body = body.try_next().await?.unwrap_or_default();
    let request = ::http::Request::from_parts(parts, body);

    // handle request by salvo
    let scheme = request.uri().scheme().cloned().unwrap_or(http::uri::Scheme::HTTP);
    let request = salvo_core::Request::from_hyper(request, scheme);
    let service = salvo_core::Service::new(router);
    let mut depot = salvo_core::Depot::new();
    depot.inject(env);
    depot.inject(ctx);
    let response = service.handle(request, Some(depot)).await;

    // parse response
    let response = crate::response::handle_response(response).await?;

    Ok(response)
}
