use salvo_worker::salvo::*;
use time::OffsetDateTime;

// Handler for short-lived cached response (5 seconds)
#[handler]
pub async fn short() -> String {
    format!("short Hello World, my birth time is {}", OffsetDateTime::now_utc())
}

// Handler for long-lived cached response (1 minute)
#[handler]
pub async fn long() -> String {
    format!("long Hello World, my birth time is {}", OffsetDateTime::now_utc())
}

// Handler for long-lived cached response (1 minute)
#[handler]
pub async fn idle_cache() -> String {
    format!("idle Hello World, my birth time is {}", OffsetDateTime::now_utc())
}
