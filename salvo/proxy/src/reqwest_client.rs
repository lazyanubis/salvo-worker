#[allow(unused)]
use futures_util::TryStreamExt;
use hyper::upgrade::OnUpgrade;
use reqwest::Client as InnerClient;
#[allow(unused)]
use salvo_core::Error;
#[allow(unused)]
use salvo_core::http::{ResBody, StatusCode};
// #[cfg(not(target_arch = "wasm32"))]
// use salvo_core::rt::tokio::TokioIo;
#[allow(unused)]
#[cfg(not(target_arch = "wasm32"))]
use tokio::io::copy_bidirectional;

use crate::{BoxedError, Client, HyperRequest, HyperResponse, Proxy, Upstreams};

/// A [`Client`] implementation based on [`reqwest::Client`].
///
/// This client provides proxy capabilities using the Reqwest HTTP client.
/// It supports all features of Reqwest including automatic redirect handling,
/// connection pooling, and other HTTP client features.
#[derive(Default, Clone, Debug)]
pub struct ReqwestClient {
    #[allow(unused)]
    inner: InnerClient,
}

impl<U> Proxy<U, ReqwestClient>
where
    U: Upstreams,
    U::Error: Into<BoxedError>,
{
    /// Create a new `Proxy` using the default Reqwest client.
    ///
    /// This is a convenient way to create a proxy with standard configuration.
    pub fn use_reqwest_client(upstreams: U) -> Self {
        Proxy::new(upstreams, ReqwestClient::default())
    }
}

impl ReqwestClient {
    /// Create a new `ReqwestClient` with the given [`reqwest::Client`].
    pub fn new(inner: InnerClient) -> Self {
        Self { inner }
    }
}

impl Client for ReqwestClient {
    type Error = salvo_core::Error;

    #[allow(unused)]
    #[cfg(not(target_arch = "wasm32"))]
    async fn execute(
        &self,
        proxied_request: HyperRequest,
        request_upgraded: Option<OnUpgrade>,
    ) -> Result<HyperResponse, Self::Error> {
        todo!()
        // let request_upgrade_type = crate::get_upgrade_type(proxied_request.headers()).map(|s| s.to_owned());

        // let proxied_request =
        //     proxied_request.map(|s| reqwest::Body::wrap_stream(s.map_ok(|s| s.into_data().unwrap_or_default())));
        // let response = self
        //     .inner
        //     .execute(proxied_request.try_into().map_err(Error::other)?)
        //     .await
        //     .map_err(Error::other)?;

        // let res_headers = response.headers().clone();
        // let hyper_response = hyper::Response::builder()
        //     .status(response.status())
        //     .version(response.version());

        // let mut hyper_response = if response.status() == StatusCode::SWITCHING_PROTOCOLS {
        //     let response_upgrade_type = crate::get_upgrade_type(response.headers());

        //     if request_upgrade_type == response_upgrade_type.map(|s| s.to_lowercase()) {
        //         let mut response_upgraded = response
        //             .upgrade()
        //             .await
        //             .map_err(|e| Error::other(format!("response does not have an upgrade extension. {}", e)))?;
        //         if let Some(request_upgraded) = request_upgraded {
        //             tokio::spawn(async move {
        //                 match request_upgraded.await {
        //                     Ok(request_upgraded) => {
        //                         let mut request_upgraded = TokioIo::new(request_upgraded);
        //                         if let Err(e) = copy_bidirectional(&mut response_upgraded, &mut request_upgraded).await
        //                         {
        //                             tracing::error!(error = ?e, "coping between upgraded connections failed");
        //                         }
        //                     }
        //                     Err(e) => {
        //                         tracing::error!(error = ?e, "upgrade request failed");
        //                     }
        //                 }
        //             });
        //         } else {
        //             return Err(Error::other("request does not have an upgrade extension"));
        //         }
        //     } else {
        //         return Err(Error::other("upgrade type mismatch"));
        //     }
        //     hyper_response.body(ResBody::None).map_err(Error::other)?
        // } else {
        //     hyper_response
        //         .body(ResBody::stream(response.bytes_stream()))
        //         .map_err(Error::other)?
        // };
        // *hyper_response.headers_mut() = res_headers;
        // Ok(hyper_response)
    }

    #[cfg(target_arch = "wasm32")]
    #[worker::send]
    async fn execute(
        &self,
        mut proxied_request: HyperRequest,
        _request_upgraded: Option<OnUpgrade>,
    ) -> Result<HyperResponse, Self::Error> {
        // let request_upgrade_type = crate::get_upgrade_type(proxied_request.headers()).map(|s| s.to_owned());

        let proxied_request = {
            use futures::stream::TryStreamExt;
            let body = proxied_request.body_mut();
            let body = body.try_next().await?;
            let body: Option<bytes::Bytes> = match body {
                Some(body) => {
                    let body = body.into_data().unwrap_or_default();
                    Some(body.into())
                }
                None => None,
            };
            proxied_request.map(|_| body.unwrap_or_default())
        };

        let response = self
            .inner
            .execute(proxied_request.try_into().map_err(Error::other)?)
            .await
            .map_err(Error::other)?;

        let res_headers = response.headers().clone();
        let hyper_response = hyper::Response::builder().status(response.status());

        let mut hyper_response = if response.status() == StatusCode::SWITCHING_PROTOCOLS {
            hyper_response.body(ResBody::None).map_err(Error::other)?
        } else {
            let bytes = response.bytes().await.map_err(Error::other)?;
            hyper_response.body(ResBody::Once(bytes.into())).map_err(Error::other)?
        };
        *hyper_response.headers_mut() = res_headers;
        Ok(hyper_response)
    }
}

// Unit tests for Proxy
#[cfg(test)]
mod tests {
    use salvo_core::prelude::*;
    use salvo_core::test::*;

    use super::*;
    use crate::{Proxy, Upstreams};

    #[tokio::test]
    async fn test_upstreams_elect() {
        let upstreams = vec!["https://www.example.com", "https://www.example2.com"];
        let proxy = Proxy::new(upstreams.clone(), ReqwestClient::default());
        let elected_upstream = proxy.upstreams().elect().await.unwrap();
        assert!(upstreams.contains(&elected_upstream));
    }

    #[ignore]
    #[tokio::test]
    async fn test_reqwest_client() {
        let router = Router::new().push(
            Router::with_path("rust/{**rest}")
                .goal(Proxy::new(vec!["https://www.rust-lang.org"], ReqwestClient::default())),
        );

        let content = TestClient::get("http://127.0.0.1:5801/rust/tools/install")
            .send(router)
            .await
            .take_string()
            .await
            .unwrap();
        assert!(content.contains("Install Rust"));
    }

    #[test]
    fn test_others() {
        let mut handler = Proxy::new(["https://www.bing.com"], ReqwestClient::default());
        assert_eq!(handler.upstreams().len(), 1);
        assert_eq!(handler.upstreams_mut().len(), 1);
    }
}
