#![doc = include_str!("../README.md")]
#![deny(unreachable_pub)] // ! lib 需要检查此项
#![deny(unsafe_code)] // 拒绝 unsafe 代码
#![deny(missing_docs)] // ! 必须写文档
#![warn(rustdoc::broken_intra_doc_links)] // 文档里面的链接有效性
#![warn(clippy::future_not_send)] // 异步代码关联的对象必须是 Send 的
#![deny(clippy::unwrap_used)] // 不许用 unwrap
#![deny(clippy::expect_used)] // 不许用 expect
#![deny(clippy::panic)] // 不许用 panic

/// test add
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}
