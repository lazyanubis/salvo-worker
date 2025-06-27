use crate::salvo::*;

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

    /// 出现错误
    #[inline]
    pub fn failed(code: u16, message: impl Into<String>) -> Self {
        MessageResponse {
            code,
            message: message.into(),
            created: now_format_utc(),
            data: None,
        }
    }

    /// bad request
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::failed(400, message)
    }
}

impl MessageResponse<()> {
    /// 成功操作
    #[inline]
    pub fn none_success() -> Self {
        Self::success()
    }

    /// 出现错误
    #[inline]
    pub fn none_failed(code: u16, message: impl Into<String>) -> Self {
        MessageResponse {
            code,
            message: message.into(),
            created: now_format_utc(),
            data: None,
        }
    }

    /// bad request
    pub fn none_bad_request(message: impl Into<String>) -> Self {
        Self::failed(400, message)
    }
}

#[cfg(feature = "oapi")]
#[async_trait]
impl<T: Serialize + Send> Writer for MessageResponse<T> {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        #[allow(clippy::unwrap_used)] // ? checked
        let result = serde_json::to_string(&self).unwrap();
        res.render(Text::Json(result));
    }
}

#[cfg(feature = "oapi")]
impl<T: Serialize + Send + ToSchema + 'static> ToSchema for MessageResponse<T> {
    fn to_schema(components: &mut oapi::Components) -> oapi::RefOr<oapi::schema::Schema> {
        use oapi::{BasicType, KnownFormat, Object, SchemaFormat, SchemaType};
        Object::new()
            .property(
                "code",
                Object::new()
                    .schema_type(SchemaType::Basic(BasicType::Integer))
                    .format(SchemaFormat::KnownFormat(KnownFormat::UInt16))
                    .description("消息码。0 表示成功，其他数字表示失败。"),
            )
            .required("code")
            .property(
                "message",
                Object::new()
                    .schema_type(SchemaType::Basic(BasicType::String))
                    .description("消息提示。成功固定为 success，错误会提示具体错误内容。"),
            )
            .required("message")
            .property(
                "created",
                Object::new()
                    .schema_type(SchemaType::Basic(BasicType::String))
                    .description("消息响应时间"),
            )
            .required("created")
            .property("data", T::to_schema(components))
            .required("data")
            .example(serde_json::json!({
                "code": 0,
                "message": "success",
                "created": "2025-05-07T03:35:44.456Z",
                "data": 123456
            }))
            .example(serde_json::json!({
                "code": 1,
                "message": "System error",
                "created": "2025-05-07T03:35:44.456Z",
            }))
            .into()
    }
}

#[cfg(feature = "oapi")]
impl<T: Serialize + Send + ToSchema + 'static> EndpointOutRegister for MessageResponse<T> {
    fn register(components: &mut oapi::Components, operation: &mut oapi::Operation) {
        operation.responses.insert(
            StatusCode::OK.as_str(),
            oapi::Response::new("请求成功")
                .add_content("application/json", MessageResponse::<T>::to_schema(components)),
        );
    }
}
