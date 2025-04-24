use worker::*;

/// 获取持久化对象
pub fn get_do_binding(env: &Env, binding: &str, path: &str) -> Result<Stub> {
    let namespace = env.durable_object(binding)?;
    namespace.id_from_name(path)?.get_stub()
}
