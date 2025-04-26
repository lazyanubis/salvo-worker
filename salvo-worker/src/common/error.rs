use std::fmt::Display;

/// 简化错误
pub fn worker_error<E: Display>(err: E) -> worker::Error {
    worker::Error::RustError(err.to_string())
}
