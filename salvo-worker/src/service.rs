use std::sync::Arc;

use futures::stream::TryStreamExt;
use worker::*;

/// service
pub struct WorkerService {
    service: salvo_core::Service,
}

impl WorkerService {
    /// 新建
    pub fn new(service: salvo_core::Service) -> Self {
        Self { service }
    }

    /// 路由
    pub fn from_router(router: Arc<salvo_core::Router>) -> Self {
        Self {
            service: salvo_core::Service::new(router),
        }
    }

    /// hoop
    pub fn hoop<H: salvo_core::Handler>(self, hoop: H) -> Self {
        Self {
            service: self.service.hoop(hoop),
        }
    }

    /// cors
    #[cfg(feature = "cors")]
    pub fn cors(self, cors: super::salvo::cors::CorsHandler) -> Self {
        Self {
            service: self.service.hoop(cors),
        }
    }

    /// cors
    #[cfg(feature = "cors")]
    pub fn catch_bad_request_and_not_found(self) -> Self {
        use salvo_core::catcher::Catcher;

        Self {
            service: self.service.catcher(
                Catcher::default()
                    .hoop(super::catch::bad_request)
                    .hoop(super::catch::not_found),
            ),
        }
    }
}

impl From<Arc<salvo_core::Router>> for WorkerService {
    fn from(value: Arc<salvo_core::Router>) -> Self {
        Self::from_router(value)
    }
}

impl WorkerService {
    /// 处理请求
    pub async fn handle(&self, req: Request, env: Env, ctx: Context) -> worker::Result<Response> {
        // parse request
        let request: HttpRequest = req.try_into()?;
        let (parts, mut body) = request.into_parts();
        let body = body.try_next().await?.unwrap_or_default();
        let request = ::http::Request::from_parts(parts, body);
        let scheme = request
            .headers()
            .iter()
            .find(|(name, _)| name.as_str() == "cf-visitor")
            .and_then(|(_, value)| value.to_str().ok())
            .and_then(|v| match v {
                r#"{"scheme":"https"}"# => Some(http::uri::Scheme::HTTPS),
                r#"{"scheme":"http"}"# => Some(http::uri::Scheme::HTTP),
                _ => None,
            });

        // handle request by salvo
        let scheme = request
            .uri()
            .scheme()
            .cloned()
            .unwrap_or_else(|| scheme.unwrap_or(http::uri::Scheme::HTTP));
        let request = salvo_core::Request::from_hyper(request, scheme);
        let mut depot = salvo_core::Depot::new();
        depot.inject(env);
        depot.inject(ctx);
        let response = self.service.handle(request, Some(depot)).await;

        // parse response
        let response = crate::response::handle_response(response).await?;

        Ok(response)
    }
}
