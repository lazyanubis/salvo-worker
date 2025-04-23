use std::time::Duration;

use salvo_worker::salvo::*;

#[handler]
pub async fn fast() -> &'static str {
    "hello"
}

#[handler]
pub async fn slow() -> &'static str {
    salvo::timeout::sleep(Duration::from_secs(10)).await;
    "hello"
}
