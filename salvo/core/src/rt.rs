//! Runtime module.
//!
//! Only supports tokio runtime in current version.
//! More runtimes support may be added in future releases.

#[doc(hidden)]
#[allow(unused)]
pub(crate) use hyper::rt::*;

/// Tokio runtimes
#[allow(unused)]
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod tokio {
    pub(crate) use hyper_util::rt::{TokioExecutor, TokioIo};
}
