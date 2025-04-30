#[allow(unused)]
use std::time::{Duration, Instant};

use salvo_core::http::{Request, Response};
use salvo_core::{Depot, FlowCtrl, Handler, async_trait};

/// elapsed
#[derive(Default, Debug)]
pub struct Elapsed {}
impl Elapsed {
    /// new
    #[inline]
    pub fn new() -> Self {
        Elapsed {}
    }
}

#[async_trait]
impl Handler for Elapsed {
    async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
        let now = worker::js_sys::Date::now();
        ctrl.call_next(req, depot, res).await;
        let end = worker::js_sys::Date::now();
        let elapsed = Duration::from_millis((end - now) as u64);
        let _ = res.add_header("X-ELAPSED-MS", elapsed.as_millis().to_string(), false);
        worker::console_debug!("elapsed: {} ms", elapsed.as_millis());
    }
}

// #[cfg(not(target_arch = "wasm32"))]
// #[cfg(test)]
// mod tests {
//     use salvo_core::prelude::*;
//     use salvo_core::test::{ResponseExt, TestClient};
//     use tracing_test::traced_test;

//     use super::*;

//     #[tokio::test]
//     #[traced_test]
//     async fn test_log() {
//         #[handler]
//         async fn hello() -> &'static str {
//             "hello"
//         }

//         let router = Router::new()
//             .hoop(Logger::new())
//             .push(Router::with_path("hello").get(hello));

//         TestClient::get("http://127.0.0.1:5801/hello")
//             .send(router)
//             .await
//             .take_string()
//             .await
//             .unwrap();
//         assert!(logs_contain("duration"));
//     }
// }
