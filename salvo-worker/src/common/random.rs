use std::future::Future;

/// 随机 id
pub async fn random_special_id<F, R>(valid: F) -> Result<i64, getrandom::Error>
where
    F: Fn(i64) -> R + Send,
    R: Future<Output = Result<bool, getrandom::Error>> + Send,
{
    loop {
        let mut id = random_i64()?;

        id = match is_valid_id(id) {
            Some(id) => id,
            None => continue,
        };

        if valid(id).await? {
            return Ok(id);
        }
    }
}

/// 随机数
#[inline]
pub fn random_u64() -> Result<u64, getrandom::Error> {
    let mut buf = [0u8; 8];
    getrandom::getrandom(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

/// 随机数
#[inline]
pub fn random_i64() -> Result<i64, getrandom::Error> {
    let mut buf = [0u8; 8];
    getrandom::getrandom(&mut buf)?;
    Ok(i64::from_be_bytes(buf))
}

/// 随机数
#[inline]
pub fn random_f64() -> Result<f64, getrandom::Error> {
    let random = random_u64()? as f64;
    Ok(random / (u64::MAX as f64))
}

/// 随机 id
#[inline]
pub fn is_valid_id(mut id: i64) -> Option<i64> {
    // i64 19 位数字 16位数字
    // Min = -9223372036854775808,
    // Max = 9223372036854775807,
    const MAX: i64 = 9007199254740991; // Number.MAX_SAFE_INTEGER 太大 js 会丢失精度
    // const MIN: i64 = -9007199254740991; // Number.MIN_SAFE_INTEGER 太小 js 会丢失精度
    id %= MAX; // 进入有效范围

    if 10000000000 < id {
        return Some(id);
    }
    None
}

/// 随机 token
#[inline]
pub fn random_token(length: u8) -> Result<String, getrandom::Error> {
    const CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut code = String::with_capacity(length as usize);
    for _ in 0..length {
        #[allow(clippy::unwrap_used)] // ? checked
        code.push(
            CHARS
                .chars()
                .nth((random_f64()? * CHARS.len() as f64) as usize)
                .unwrap(),
        );
    }
    Ok(code)
}

/// 随机数字
#[inline]
pub fn random_code(length: u8) -> Result<String, getrandom::Error> {
    let mut code = String::new();
    for _ in 0..length {
        #[allow(clippy::unwrap_used)] // ? checked
        code.push(char::from_u32((random_f64()? * 10_f64) as u32 + 48).unwrap());
    }
    Ok(code)
}
