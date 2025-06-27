use futures::stream::TryStreamExt;
use worker::*;

#[inline]
pub(crate) async fn handle_response(mut response: salvo_core::Response) -> worker::Result<Response> {
    let mut body = response.body.take();
    let body = body.try_next().await?;
    let body = match body {
        Some(body) => {
            let body = body
                .into_data()
                .map_err(|err| worker::Error::Json((format!("can not get body bytes: {err:?}"), 1)))?;
            ResponseBody::Body(body.into())
        }
        None => ResponseBody::Empty,
    };

    let headers = worker::Headers::new();
    for (name, value) in response.headers {
        if let Some(name) = name {
            match value.to_str() {
                Ok(value) => headers.append(name.as_str(), value)?,
                Err(err) => {
                    return Err(worker::Error::Json((format!("can not get header value: {err:?}"), 1)));
                }
            };
        }
    }

    #[cfg(feature = "cookie")]
    {
        use http::header::HeaderValue;
        for cookie in response.cookies.delta() {
            let hv = HeaderValue::from_str(&cookie.encoded().to_string());
            if let Ok(hv) = hv {
                if let Ok(hv) = hv.to_str() {
                    headers.append(::http::header::SET_COOKIE.as_str(), hv)?;
                }
            }
        }
    }

    let builder = Response::builder()
        .with_status(response.status_code.map(|code| code.as_u16()).unwrap_or(200))
        .with_headers(headers)
        .body(body);

    Ok(builder)
}
