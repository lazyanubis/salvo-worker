use salvo_worker::salvo::*;

// Custom validator implementing BasicAuthValidator trait
pub(crate) struct Validator;
impl basic_auth::BasicAuthValidator for Validator {
    // Validate username and password combination
    async fn validate(&self, username: &str, password: &str, _depot: &mut Depot) -> bool {
        username == "root" && password == "pwd"
    }
}

// Simple handler that returns "Hello" for authenticated requests
#[handler]
pub(crate) async fn hello() -> &'static str {
    "Hello"
}
