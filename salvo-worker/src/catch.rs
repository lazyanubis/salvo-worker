use salvo_core::http::ResBody;

use crate::common::response::MessageResponse;

use super::salvo::*;

#[handler]
pub async fn bad_request(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if let ResBody::Error(StatusError {
        code,
        // name,
        // brief,
        // detail,
        cause,
        ..
    }) = &res.body
        && *code == StatusCode::BAD_REQUEST
        && let Some(cause) = cause
    {
        let msg = cause.to_string();
        res.status_code(StatusCode::OK);
        let response = MessageResponse::<()>::failed(400, format!("{} {} {}", req.method(), req.uri().path(), msg));
        response.write(req, depot, res).await;
    }
}

#[handler]
pub async fn not_found(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if let Some(StatusCode::NOT_FOUND) = res.status_code {
        res.status_code(StatusCode::OK);
        let response = MessageResponse::<()>::failed(404, format!("{} {}", req.method(), req.uri().path()));
        response.write(req, depot, res).await;
    }
}
