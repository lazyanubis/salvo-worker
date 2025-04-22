//! 主入口

use worker::*;

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
    let _ = router::ROUTER.clone();
}

/// 定时任务
#[event(scheduled)]
pub async fn scheduled(_event: ScheduledEvent, _env: Env, _ctx: ScheduleContext) {
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

    let router = router::ROUTER.clone();
    salvo_worker::handle(router, req, env, ctx).await
}
