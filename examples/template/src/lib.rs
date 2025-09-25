//! 主入口

use worker::*;

mod durable;
mod router;

// 初始化任务
#[event(start)]
fn start() {
    console_error_panic_hook::set_once();

    use tracing_subscriber::fmt::format::Pretty;
    use tracing_subscriber::fmt::time::UtcTime;
    use tracing_subscriber::prelude::*;
    use tracing_web::{MakeConsoleWriter, performance_layer};

    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false) // Only partially supported across JavaScript runtime
        .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
        .with_writer(MakeConsoleWriter); // write events to the console
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());
    tracing_subscriber::registry().with(fmt_layer).with(perf_layer).init();

    // 初始化
    let _ = router::WORKER_SERVICE.clone();
}

#[allow(clippy::future_not_send)]
mod future_warning {
    use super::*;

    /// 定时任务
    #[event(scheduled)]
    async fn scheduled(_event: ScheduledEvent, _env: Env, _ctx: ScheduleContext) {
        // // console_error_panic_hook::set_once();
        // initial::do_init(&env).await; // 初始化

        // 执行任务
        // handlers::triggers::cron_trigger_handler(_event, env, _ctx).await;
    }

    /// 请求
    #[event(fetch)]
    async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
        // // console_error_panic_hook::set_once();
        // initial::do_init(&env).await; // 初始化

        let path = req.path();
        if path.starts_with("/durable/websocket2") {
            let upgrade_header = req.headers().get("Upgrade")?;
            if upgrade_header.is_none_or(|upgrade_header| upgrade_header != "websocket") {
                return Response::from_bytes("Durable Object expected Upgrade: websocket".into())
                    .map(|r| r.with_status(426));
            }

            return salvo_worker::durable::get_do_binding(&env, "MY_DURABLE_OBJECT_WEB_SOCKET2", "socket")?
                .fetch_with_request(req)
                .await;
        }
        if path.starts_with("/durable/websocket") {
            let upgrade_header = req.headers().get("Upgrade")?;
            if upgrade_header.is_none_or(|upgrade_header| upgrade_header != "websocket") {
                return Response::from_bytes("Durable Object expected Upgrade: websocket".into())
                    .map(|r| r.with_status(426));
            }

            return salvo_worker::durable::get_do_binding(&env, "MY_DURABLE_OBJECT_WEB_SOCKET", "socket")?
                .fetch_with_request(req)
                .await;
        }
        if path.starts_with("/durable") {
            return salvo_worker::durable::get_do_binding(&env, "MY_DURABLE_OBJECT", "durable")?
                .fetch_with_request(req)
                .await;
        }

        router::WORKER_SERVICE.handle(req, env, ctx).await
    }
}
