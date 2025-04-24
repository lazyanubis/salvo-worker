use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::time::now_format;

/// 发送通知错误
#[derive(Debug, Error)]
pub enum LarkNoticeError {
    /// reqwest 错误
    #[error("reqwest error: {0}")]
    Reqwest(reqwest::Error),
    /// lark 服务错误
    #[error("{0}")]
    Lark(String),
}

impl From<reqwest::Error> for LarkNoticeError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

#[inline]
async fn inner_send_lark_notice(url: &str, text: &str) -> Result<(), LarkNoticeError> {
    let client = reqwest::Client::new();

    #[derive(Serialize)]
    struct LarkContent<'a> {
        text: &'a str,
    }

    #[derive(Serialize)]
    struct LarkBody<'a> {
        msg_type: &'a str,
        content: LarkContent<'a>,
    }

    let body: LarkBody = LarkBody {
        msg_type: "text",
        content: LarkContent {
            text: &format!("{text}\n{}", now_format()),
        },
    };

    #[derive(Deserialize, Serialize, Debug)]
    struct LarkResult {
        code: u32,
        msg: String,
    }

    let json = client.post(url).json(&body).send().await?.json::<LarkResult>().await?;

    if json.code != 0 {
        tracing::info!("lark notice service failed: {:?}", json);
        return Err(LarkNoticeError::Lark(format!(
            "lark notice service failed: {}",
            json.msg
        )));
    }

    Ok(())
}

/// 发送 lark 通知
#[inline]
pub async fn send_lark(url: &str, text: &str) -> Result<(), LarkNoticeError> {
    let result = inner_send_lark_notice(url, text).await;
    if let Err(err) = &result {
        tracing::error!("send notice error: {err:?}\n{text}");
    }
    result
}
