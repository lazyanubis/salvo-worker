use salvo_worker::salvo::*;

#[handler]
pub(crate) async fn hello() {
    #[allow(clippy::panic)]
    {
        panic!("panic error!");
    }
}
