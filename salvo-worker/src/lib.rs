#![doc = include_str!("../../README.md")]
#![deny(unreachable_pub)] // ! lib 需要检查此项
#![deny(unsafe_code)] // 拒绝 unsafe 代码
#![deny(missing_docs)] // ! 必须写文档
#![warn(rustdoc::broken_intra_doc_links)] // 文档里面的链接有效性
#![warn(clippy::future_not_send)] // 异步代码关联的对象必须是 Send 的
#![deny(clippy::unwrap_used)] // 不许用 unwrap
#![deny(clippy::expect_used)] // 不许用 expect
#![deny(clippy::panic)] // 不许用 panic

/// common
pub mod common;

/// durable
pub mod durable;

/// response
mod response;

/// open_api
#[cfg(feature = "oapi")]
pub mod open_api;

/// service
mod service;
pub use service::*;

/// 导出所有 salvo 需要用到的
pub mod salvo {
    pub use ::serde::{Deserialize, Serialize};
    pub use salvo_core::prelude::*;
    pub use salvo_core::*;

    /// 导出所有 salvo 需要用到的
    #[allow(clippy::module_inception)]
    pub mod salvo {
        pub use super::*;
    }

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

    // #[cfg(feature = "compression")]
    // pub use salvo_compression as compression;

    #[cfg(feature = "concurrency-limiter")]
    pub use salvo_extra::concurrency_limiter;

    #[cfg(feature = "cors")]
    pub use salvo_cors as cors;

    #[cfg(feature = "session")]
    pub use salvo_session as session;

    #[cfg(feature = "csrf")]
    pub use salvo_csrf as csrf;

    #[cfg(feature = "flash")]
    pub use salvo_flash as flash;

    // #[cfg(feature = "jwt-auth")]
    // pub use salvo_jwt_auth as jwt_auth;

    #[cfg(feature = "logging")]
    pub use salvo_extra::logging;

    #[cfg(feature = "oapi")]
    pub use salvo_oapi as oapi;
    #[cfg(feature = "oapi")]
    mod oapi_prelude {
        pub use super::oapi::extract::{PathParam, QueryParam};
        pub use super::oapi::rapidoc::RapiDoc;
        pub use super::oapi::redoc::ReDoc;
        pub use super::oapi::scalar::Scalar;
        pub use super::oapi::swagger_ui::SwaggerUi;
        pub use super::oapi::{
            EndpointArgRegister, EndpointOutRegister, OpenApi, RouterExt, ToParameter, ToParameters, ToResponse,
            ToResponses, ToSchema, endpoint,
        };
    }
    #[cfg(feature = "oapi")]
    pub use oapi_prelude::*;

    #[cfg(feature = "proxy")]
    pub use salvo_proxy as proxy;

    #[cfg(feature = "rate-limiter")]
    pub use salvo_rate_limiter as rate_limiter;

    #[cfg(feature = "request-id")]
    pub use salvo_extra::request_id::RequestId;

    // #[cfg(feature = "serve-static")]
    // pub use salvo_serve_static::{StaticDir, StaticFile};

    #[cfg(feature = "size-limiter")]
    pub use salvo_extra::size_limiter;

    // #[cfg(feature = "sse")]
    // pub use salvo_extra::sse;

    #[cfg(feature = "timeout")]
    pub use salvo_extra::timeout;

    #[cfg(feature = "trailing-slash")]
    pub use salvo_extra::trailing_slash;
}
