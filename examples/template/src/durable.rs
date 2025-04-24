use worker::*;

use salvo_worker::common::response::MessageResponse;

mod web_socket;

const ENABLED: &str = "enabled";

#[durable_object]
pub struct TestTemplateDurableObject {
    #[allow(unused)]
    state: State,
    #[allow(unused)]
    env: Env,
}

#[durable_object]
impl DurableObject for TestTemplateDurableObject {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        let path = req.path();
        match path.as_str() {
            "/durable/start" => {
                self.state.storage().put(ENABLED, true).await?;
                self.assure_alarm(10000).await?;
                return Response::from_json(&MessageResponse::<()>::success());
            }
            "/durable/stop" => {
                self.state.storage().put(ENABLED, false).await?;
                self.state.storage().delete_alarm().await?;
                return Response::from_json(&MessageResponse::<()>::success());
            }
            _ => {}
        }
        Response::from_bytes("Not Found".into()).map(|v| v.with_status(404))
    }

    async fn alarm(&mut self) -> Result<Response> {
        console_debug!("Durable Object Alarm");
        self.assure_alarm(10000).await?;
        return Response::from_json(&MessageResponse::<()>::success());
    }
}

impl TestTemplateDurableObject {
    async fn assure_alarm(&self, delay_ms: i64) -> Result<Option<i64>> {
        let enabled: bool = self.state.storage().get(ENABLED).await?;
        let alarm = self.state.storage().get_alarm().await?;
        match (enabled, alarm) {
            (false, Some(_)) => self.state.storage().delete_alarm().await?,
            (false, None) => {}
            (true, None) => {
                let now = worker::js_sys::Date::now() as i64;
                let alarm = now + delay_ms;
                let next = ScheduledTime::new(worker::js_sys::Date::new(&worker::wasm_bindgen::JsValue::from_f64(
                    alarm as f64,
                )));
                self.state
                    .storage()
                    .set_alarm_with_options(
                        next,
                        SetAlarmOptions {
                            allow_concurrency: Some(false),
                            allow_unconfirmed: Some(false),
                        },
                    )
                    .await?;
                return Ok(Some(alarm));
            }
            (true, Some(alarm)) => return Ok(Some(alarm)),
        }
        Ok(None)
    }
}
