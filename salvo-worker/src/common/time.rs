use ::time::{OffsetDateTime, UtcOffset};

/// 秒 ms
pub const SECOND: i64 = 1000;
/// 分钟 ms
pub const MINUTE: i64 = SECOND * 60;
/// 小时 ms
pub const HOUR: i64 = MINUTE * 60;
/// 天 ms
pub const DAY: i64 = HOUR * 24;

#[inline]
fn now() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

/// 获取当前时间戳（毫秒）
#[inline]
pub fn now_ms() -> i64 {
    worker::js_sys::Date::now() as i64
}

/// 获取当前时间戳（纳秒）
#[inline]
pub fn now_nanos() -> i64 {
    worker::js_sys::Date::now() as i64 * 1_000_000
}

/// 格式化时间，UTC 日期时间
#[inline]
pub fn now_format_utc() -> String {
    let now = now();
    let date = now.date();
    let time = now.time();
    let mills = now.millisecond();
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        date.year(),
        date.month() as u8,
        date.day(),
        time.hour(),
        time.minute(),
        time.second(),
        mills
    )
}

/// 格式化时间，UTC+8 日期时间
#[inline]
pub fn now_format() -> String {
    #[allow(clippy::unwrap_used)] // ? SAFETY
    let offset = UtcOffset::from_hms(8, 0, 0).unwrap();
    let now = now().to_offset(offset);
    let date = now.date();
    let time = now.time();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        date.year(),
        date.month() as u8,
        date.day(),
        time.hour(),
        time.minute(),
        time.second()
    )
}

/// 格式化时间 上海时间
#[allow(unused)]
#[inline]
pub fn format_date_time(nanos: i128) -> String {
    #[allow(clippy::unwrap_used)] // ? SAFETY
    let now = OffsetDateTime::from_unix_timestamp_nanos(nanos).unwrap();
    let date = now.date();
    let time = now.time();
    let mills = now.millisecond();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}+8",
        date.year(),
        date.month() as u8,
        date.day(),
        time.hour(),
        time.minute(),
        time.second(),
        mills
    )
}
