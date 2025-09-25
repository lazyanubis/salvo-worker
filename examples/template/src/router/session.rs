use salvo_worker::salvo::*;

use session::{Session, SessionDepotExt};

#[handler]
pub(crate) async fn login(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if req.method() == salvo::http::Method::POST {
        let mut session = Session::new();
        #[allow(clippy::unwrap_used)]
        let username = req.form::<String>("username").await.unwrap();
        #[allow(clippy::unwrap_used)]
        session.insert("username", username).unwrap();
        depot.set_session(session);
        res.render(Redirect::other("/session"));
    } else {
        res.render(Text::Html(LOGIN_HTML));
    }
}

#[handler]
pub(crate) async fn logout(depot: &mut Depot, res: &mut Response) {
    if let Some(session) = depot.session_mut() {
        session.remove("username");
    }
    res.render(Redirect::other("/"));
}

#[handler]
pub(crate) async fn home(depot: &mut Depot, res: &mut Response) {
    let mut content = r#"<a href="/session/login">Login</h1>"#.into();
    if let Some(session) = depot.session_mut()
        && let Some(username) = session.get::<String>("username")
    {
        content = format!(r#"Hello, {username}. <br><a href="logout">Logout</h1>"#);
    }
    res.render(Text::Html(content));
}

static LOGIN_HTML: &str = r#"<!DOCTYPE html>
<html>
    <head>
        <title>Login</title>
    </head>
    <body>
        <form action="/session/login" method="post">
            <h1>Login</h1>
            <input type="text" name="username" />
            <button type="submit" id="submit">Submit</button>
        </form>
    </body>
</html>
"#;
