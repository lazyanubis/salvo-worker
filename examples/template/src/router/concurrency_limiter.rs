use salvo_worker::salvo::*;

// Handler for serving the index page with upload forms
#[handler]
pub(crate) async fn index() -> &'static str {
    "123"
}
