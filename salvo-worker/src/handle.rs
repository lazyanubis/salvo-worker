use futures::stream::TryStreamExt;
use salvo_core::test::SendTarget;
use worker::*;

mod inner {
    use salvo_core::prelude::*;
    use serde::Serialize;

    #[handler]
    async fn hello() -> Result<&'static str, ()> {
        Ok("Hello World")
    }
    #[handler]
    async fn json(res: &mut Response) {
        #[derive(Serialize, Debug)]
        struct User {
            name: String,
        }
        res.render(Json(User { name: "jobs".into() }));
    }

    pub(super) fn get_router() -> Router {
        Router::new()
            .get(hello)
            .push(Router::with_path("json").get(json))
            .push(Router::with_path("hello").get(json))
    }
}

/// handle request
pub async fn handle(req: Request, _env: Env, _ctx: Context) -> worker::Result<Response> {
    // let router = Router::new();
    // router
    //     .get_async("/hello", |_, _| async move { Response::ok("Hello, World!") })
    //     .run(req, env)
    //     .await

    let request: HttpRequest = req.try_into()?;
    let (parts, mut body) = request.into_parts();
    let body = body.try_next().await?.unwrap_or_default();
    let request = ::http::Request::from_parts(parts, body);

    let scheme = request.uri().scheme().cloned().unwrap_or(http::uri::Scheme::HTTP);
    let request = salvo_core::Request::from_hyper(request, scheme);
    let router = inner::get_router();
    let service = salvo_core::Service::new(router);

    let response = service.call(request).await;
    let response = handle_response(response).await?;

    Ok(response)
}

async fn handle_response(mut response: salvo_core::Response) -> worker::Result<Response> {
    let mut headers = worker::Headers::new();
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

    let builder = Response::builder()
        .with_status(response.status_code.map(|code| code.as_u16()).unwrap_or(200))
        .with_headers(headers)
        .body(body);

    Ok(builder.into())
}
