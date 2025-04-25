use std::sync::Arc;

use futures::StreamExt;
use tokio::sync::RwLock;
use worker::*;

struct InnerState {
    #[allow(unused)]
    state: State,
    #[allow(unused)]
    env: Env,

    currently_connected_web_sockets: u64,
}

#[durable_object]
pub struct TestTemplateWebSocketDurableObject {
    inner: Arc<RwLock<InnerState>>,
}

#[durable_object]
impl DurableObject for TestTemplateWebSocketDurableObject {
    fn new(state: State, env: Env) -> Self {
        Self {
            inner: Arc::new(RwLock::new(InnerState {
                state,
                env,
                currently_connected_web_sockets: 0,
            })),
        }
    }

    #[allow(clippy::unwrap_used)]
    async fn fetch(&mut self, _req: Request) -> Result<Response> {
        let pair = WebSocketPair::new()?;
        let WebSocketPair { client, server } = pair;

        server.accept()?;
        server.send_with_str("Hello, world!")?;
        self.inner.write().await.currently_connected_web_sockets += 1;

        let mut this = Self {
            inner: self.inner.clone(),
        };
        worker::wasm_bindgen_futures::spawn_local(async move {
            if let Err(err) = this.handle_socket(server).await {
                console_error!("WebSocket error: {:?}", err);
            }
        });

        Response::from_websocket(client)
    }
}

#[allow(clippy::unwrap_used)]
impl TestTemplateWebSocketDurableObject {
    async fn handle_socket(&mut self, server: WebSocket) -> Result<()> {
        let mut event_steam = server.events()?;
        while let Some(Ok(event)) = event_steam.next().await {
            match event {
                WebsocketEvent::Message(message) => {
                    let got = message.text().unwrap_or_default();
                    server.send_with_str(format!(
                        "got: {got} send: {}",
                        self.inner.read().await.currently_connected_web_sockets
                    ))?;
                }
                WebsocketEvent::Close(event) => {
                    self.inner.write().await.currently_connected_web_sockets -= 1;
                    server.close(Some(event.code()), Some("Durable Object is closing WebSocket"))?;
                    break;
                }
            }
        }
        Ok(())
    }
}
