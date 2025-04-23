use salvo_worker::salvo::{csrf::CsrfDepotExt, *};

// Handler for serving the home page with links to different CSRF protection methods
#[handler]
pub async fn home(res: &mut Response) {
    let html = r#"
<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"><title>Csrf CookieStore</title></head>
<body>
<h2>Csrf Example: CookieStore</h2>
<ul>
    <li><a href="/csrf/bcrypt/">Bcrypt</a></li>
    <li><a href="/csrf/hmac/">Hmac</a></li>
    <li><a href="/csrf/aes_gcm/">Aes Gcm</a></li>
    <li><a href="/csrf/ccp/">chacha20poly1305</a></li>
</ul>
</body>"#;
    res.render(Text::Html(html));
}

// Handler for GET requests that displays a form with CSRF token
#[handler]
pub async fn get_page(depot: &mut Depot, res: &mut Response) {
    let new_token = depot.csrf_token().unwrap_or_default();
    res.render(Text::Html(get_page_html(new_token, "")));
}

// Handler for POST requests that processes form submission with CSRF validation
#[handler]
pub async fn post_page(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    // Define data structure for form submission
    #[derive(Deserialize, Serialize, Debug)]
    struct Data {
        csrf_token: String,
        message: String,
    }
    // Parse the submitted form data into the Data struct
    #[allow(clippy::unwrap_used)]
    let data = req.parse_form::<Data>().await.unwrap();
    // Log the received form data for debugging
    tracing::info!("posted data: {:?}", data);
    // Generate a new CSRF token for the next request
    let new_token = depot.csrf_token().unwrap_or_default();
    // Generate HTML response with the new token and display the submitted data
    let html = get_page_html(new_token, &format!("{data:#?}"));
    // Send the HTML response back to the client
    res.render(Text::Html(html));
}

// Helper function to generate HTML page with CSRF token and message
fn get_page_html(csrf_token: &str, msg: &str) -> String {
    format!(
        r#"
    <!DOCTYPE html>
    <html>
    <head><meta charset="UTF-8"><title>Csrf Example</title></head>
    <body>
    <h2>Csrf Example: CookieStore</h2>
    <ul>
        <li><a href="/csrf/bcrypt/">Bcrypt</a></li>
        <li><a href="/csrf/hmac/">Hmac</a></li>
        <li><a href="/csrf/aes_gcm/">Aes Gcm</a></li>
        <li><a href="/csrf/ccp/">chacha20poly1305</a></li>
    </ul>
    <form action="./" method="post">
        <input type="hidden" name="csrf_token" value="{csrf_token}" />
        <div>
            <label>Message:<input type="text" name="message" /></label>
        </div>
        <button type="submit">Send</button>
    </form>
    <pre>{msg}</pre>
    </body>
    </html>
    "#
    )
}
