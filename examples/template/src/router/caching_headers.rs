use salvo_worker::salvo::*;

#[handler]
pub(crate) async fn hello() -> &'static str {
    "Hello World"
}
