use salvo_worker::salvo::*;

// #[endpoint]
#[handler]
pub async fn hello(name: QueryParam<String, false>, age: QueryParam<u8, true>) -> String {
    format!(
        "Hello, {}! You are {} ages.",
        name.as_deref().unwrap_or("World"),
        age.into_inner()
    )
}

// #[endpoint(
//     parameters(
//         ("pet_id", Path, description = "Pet database id to get Pet for"),
//     ),
//     responses(
//         (status_code = 200, description = "Pet found successfully"),
//         (status_code = 404, description = "Pet was not found")
//     ),
//     security(
//         (),
//         ("my_auth" = ["read:items", "edit:items"]),
//         ("token_jwt" = []),
//         ("api_key1" = [], "api_key2" = []),
//     )
// )]
#[handler]
pub async fn hello2(pet_id: PathParam<u64>) -> String {
    format!("Hello, {}!", pet_id.into_inner())
}