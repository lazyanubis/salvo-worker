#![doc = include_str!("../../README.md")]
#![deny(unreachable_pub)] // ! lib 需要检查此项
#![deny(unsafe_code)] // 拒绝 unsafe 代码
#![deny(missing_docs)] // ! 必须写文档
#![warn(rustdoc::broken_intra_doc_links)] // 文档里面的链接有效性
#![warn(clippy::future_not_send)] // 异步代码关联的对象必须是 Send 的
#![deny(clippy::unwrap_used)] // 不许用 unwrap
#![deny(clippy::expect_used)] // 不许用 expect
#![deny(clippy::panic)] // 不许用 panic

/// response
mod response;

/// service
mod service;
pub use service::*;

/// 导出所有 salvo 需要用到的
pub mod salvo {
    pub use ::serde::{Deserialize, Serialize};
    pub use salvo_core::prelude::*;
    pub use salvo_core::*;

    #[cfg(feature = "affix-state")]
    pub use salvo_extra::affix_state;

    #[cfg(feature = "basic-auth")]
    pub use salvo_extra::basic_auth;

    #[cfg(feature = "cache")]
    pub use salvo_cache as cache;

    #[cfg(feature = "caching-headers")]
    pub use salvo_extra::caching_headers;

    #[cfg(feature = "catch-panic")]
    pub use salvo_extra::catch_panic;

    #[cfg(feature = "compression")]
    pub use salvo_compression as compression;

    #[cfg(feature = "concurrency-limiter")]
    pub use salvo_extra::concurrency_limiter;

    #[cfg(feature = "cors")]
    pub use salvo_cors as cors;
}
