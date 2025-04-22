use salvo_worker::salvo::*;

#[handler]
pub async fn hello() -> &'static str {
    "Hello World"
}
