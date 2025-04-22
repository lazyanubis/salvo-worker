use salvo_worker::salvo::*;

#[handler]
pub(crate) async fn hello() {
    panic!("panic error!");
}
