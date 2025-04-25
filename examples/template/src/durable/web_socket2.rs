use worker::*;

#[durable_object]
pub struct TestTemplateWebSocket2DurableObject {
    #[allow(unused)]
    state: State,
    #[allow(unused)]
    env: Env,
}

#[durable_object]
impl DurableObject for TestTemplateWebSocket2DurableObject {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    #[allow(clippy::unwrap_used)]
    async fn fetch(&mut self, _req: Request) -> Result<Response> {
        let pair = WebSocketPair::new()?;
        let WebSocketPair { client, server } = pair;

        self.state.accept_web_socket(&server);

        Response::from_websocket(client)
    }

    #[allow(unused_variables, clippy::diverging_sub_expression)]
    async fn websocket_message(&mut self, ws: WebSocket, message: WebSocketIncomingMessage) -> Result<()> {
        match message {
            WebSocketIncomingMessage::String(message) => {
                ws.send_with_str(format!("got: {message} send: {}", self.state.get_websockets().len()))?;
            }
            WebSocketIncomingMessage::Binary(message) => {
                ws.send_with_str(format!("got: {message:?} send: {}", self.state.get_websockets().len()))?;
            }
        }
        Ok(())
    }

    #[allow(unused_variables, clippy::diverging_sub_expression)]
    async fn websocket_close(&mut self, ws: WebSocket, code: usize, reason: String, was_clean: bool) -> Result<()> {
        ws.close(Some(code as u16), Some(reason))
    }

    #[allow(unused_variables, clippy::diverging_sub_expression)]
    async fn websocket_error(&mut self, ws: WebSocket, error: Error) -> Result<()> {
        console_error!("got websocket error: {}", error);
        Ok(())
    }
}
