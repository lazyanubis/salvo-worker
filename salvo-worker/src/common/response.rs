use serde::{Deserialize, Serialize};

use super::time::now_format_utc;

/// 消息对象
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse<T: Serialize + Send + 'static> {
    /// 标识码
    pub code: u16,
    /// 消息概要
    pub message: String,
    /// 消息创建时间
    #[serde(default = "default_created")]
    pub created: String,
    /// 携带的数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

fn default_created() -> String {
    now_format_utc()
}

impl<T: Serialize + Send + 'static> MessageResponse<T> {
    /// 成功操作
    #[inline]
    pub fn success() -> Self {
        MessageResponse {
            code: 0,
            message: "success".to_string(),
            created: now_format_utc(),
            data: None,
        }
    }

    /// 成功带有数据
    #[inline]
    pub fn data(data: T) -> Self {
        MessageResponse {
            code: 0,
            message: "success".to_string(),
            created: now_format_utc(),
            data: Some(data),
        }
    }
}

/// 请求结果
pub type AppResult<T, E> = Result<MessageResponse<T>, E>;

impl<T: Serialize + Send + 'static, E> From<MessageResponse<T>> for AppResult<T, E> {
    fn from(value: MessageResponse<T>) -> Self {
        Ok(value)
    }
}
