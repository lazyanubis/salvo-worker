use salvo_oapi::ToSchema;
use serde::{Deserialize, Serialize};

/// 分页请求结果
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PageData<T> {
    /// 当前过滤条件下的总数
    pub total: u64,
    /// 当前页
    pub page: u64,
    /// 每页数量
    pub size: u32,
    /// 当前分页内容列表
    pub data: Vec<T>,
}

impl<T> PageData<T> {
    /// transform
    pub fn into<R>(self) -> PageData<R>
    where
        R: From<T>,
    {
        PageData {
            total: self.total,
            page: self.page,
            size: self.size,
            data: self.data.into_iter().map(|d| d.into()).collect(),
        }
    }

    /// transform
    pub fn try_into<R, E>(self) -> Result<PageData<R>, E>
    where
        R: TryFrom<T, Error = E>,
    {
        Ok(PageData {
            total: self.total,
            page: self.page,
            size: self.size,
            data: self
                .data
                .into_iter()
                .map(|d| d.try_into())
                .collect::<Result<Vec<_>, E>>()?,
        })
    }
}
