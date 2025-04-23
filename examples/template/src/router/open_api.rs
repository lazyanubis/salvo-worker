use salvo_worker::salvo::*;

#[handler]
// #[endpoint]
pub async fn hello(// name: QueryParam<String, false>
) -> &'static str {
    // format!("Hello, {}!", name.as_deref().unwrap_or("World"))
    "Hello!"
}
