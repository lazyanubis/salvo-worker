//! A simple logging middleware.
//!
//! # Example
//!
//! ```no_run
//! use salvo_core::prelude::*;
//! use salvo_extra::logging::Logger;
//!
//!
//! #[handler]
//! async fn hello() -> &'static str {
//!     "Hello World"
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let router = Router::new().get(hello);
//!     let service = Service::new(router).hoop(Logger::new());
//!
//!     let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;
//!     Server::new(acceptor).serve(service).await;
//! }
//! ```
#[allow(unused)]
use std::time::{Duration, Instant};

use tracing::{Instrument, Level};

use salvo_core::http::{Request, ResBody, Response, StatusCode};
use salvo_core::{Depot, FlowCtrl, Handler, async_trait};

/// A simple logger middleware.
#[derive(Default, Debug)]
pub struct Logger {}
impl Logger {
    /// Create new `Logger` middleware.
    #[inline]
    pub fn new() -> Self {
        Logger {}
    }
}

#[async_trait]
impl Handler for Logger {
    async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
        #[cfg(not(target_arch = "wasm32"))]
        let span = tracing::span!(
            Level::INFO,
            "Request",
            remote_addr = %req.remote_addr().to_string(),
            version = ?req.version(),
            method = %req.method(),
            path = %req.uri(),
        );
        #[cfg(target_arch = "wasm32")]
        let span = tracing::span!(
            Level::INFO,
            "Request",
            version = ?req.version(),
            method = %req.method(),
            path = %req.uri(),
        );

        async move {
            #[cfg(not(target_arch = "wasm32"))]
            let duration = {
                let now = Instant::now();
                ctrl.call_next(req, depot, res).await;
                now.elapsed()
            };

            #[cfg(target_arch = "wasm32")]
            let duration = {
                let now = worker::js_sys::Date::now();
                ctrl.call_next(req, depot, res).await;
                let end = worker::js_sys::Date::now();
                Duration::from_millis((end - now) as u64)
            };

            let status = res.status_code.unwrap_or(match &res.body {
                ResBody::None => StatusCode::NOT_FOUND,
                ResBody::Error(e) => e.code,
                _ => StatusCode::OK,
            });
            tracing::info!(
                %status,
                ?duration,
                "Response"
            );
        }
        .instrument(span)
        .await
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
