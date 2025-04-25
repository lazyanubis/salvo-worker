use std::fmt::Display;

/// 显示日志
pub trait DisplayOption {
    /// 显示
    fn display(&self) -> String;
}

impl<T: Display> DisplayOption for Option<T> {
    fn display(&self) -> String {
        self.as_ref().map(|v| v.to_string()).unwrap_or_else(|| "None".into())
    }
}

/// 显示日志
pub trait DisplayOptionBy<T> {
    /// 显示
    fn display_by<F: Fn(&T) -> String>(&self, f: F) -> String;
}

impl<T> DisplayOptionBy<T> for Option<T> {
    fn display_by<F: Fn(&T) -> String>(&self, f: F) -> String {
        self.as_ref().map(f).unwrap_or_else(|| "None".into())
    }
}
