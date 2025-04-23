use salvo_worker::salvo::*;

#[handler]
pub async fn hello(req: &mut Request) -> String {
    format!("Request id: {:?}", req.header::<String>("x-request-id"))
}
