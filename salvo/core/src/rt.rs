//! Runtime module.
//!
//! Only supports tokio runtime in current version.
//! More runtimes support may be added in future releases.

#[doc(hidden)]
pub use hyper::rt::*;

/// Tokio runtimes
#[cfg(feature = "needless")]
pub mod tokio {
    pub use hyper_util::rt::{TokioExecutor, TokioIo};
}
