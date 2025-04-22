use salvo_worker::salvo::*;

#[endpoint]
pub async fn hello(name: QueryParam<String, false>) -> String {
    format!("Hello, {}!", name.as_deref().unwrap_or("World"))
}
