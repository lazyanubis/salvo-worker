use std::sync::Arc;
use std::sync::Mutex;

use salvo_worker::salvo::*;

// Configuration structure with username and password
#[allow(dead_code)]
#[derive(Default, Clone, Debug)]
pub(crate) struct Config {
    pub(crate) username: String,
    pub(crate) password: String,
}

// State structure to hold a list of fail messages
#[derive(Default, Debug)]
pub(crate) struct State {
    pub(crate) fails: Mutex<Vec<String>>,
}

#[handler]
pub(crate) async fn hello(depot: &mut Depot) -> String {
    // Obtain the Config instance from the depot
    #[allow(clippy::unwrap_used)]
    let config = depot.obtain::<Config>().unwrap();
    // Get custom data from the depot
    #[allow(clippy::unwrap_used)]
    let custom_data = depot.get::<&str>("custom_data").unwrap();
    // Obtain the shared State instance from the depot
    #[allow(clippy::unwrap_used)]
    let state = depot.obtain::<Arc<State>>().unwrap();
    // Lock the fails vector and add a new fail message
    #[allow(clippy::unwrap_used)]
    let mut fails_ref = state.fails.lock().unwrap();
    fails_ref.push("fail message".into());
    // Format and return the response string
    format!("Hello World\nConfig: {config:#?}\nFails: {fails_ref:#?}\nCustom Data: {custom_data}")
}
