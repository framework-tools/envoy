use envoy::http::{self, url::Url, Method};

mod test_utils;

async fn auth_middleware<'a>(
    ctx: envoy::Context<()>,
    next: envoy::Next<()>,
) -> envoy::Result {
    let authenticated = match ctx.header("X-Auth") {
        Some(header) => header == "secret_key",
        None => false,
    };
    if authenticated {
        Ok(next.run(ctx).await)
    } else {
        Ok(envoy::Response::new(envoy::StatusCode::Unauthorized))
    }
}

async fn echo_path<State>(ctx: envoy::Context<State>) -> envoy::Result<String> {
    Ok(ctx.req.url().path().to_string())
}

#[async_std::test]
async fn route_middleware() {
    let mut app = envoy::new();
    app.at("/protected").with(auth_middleware).get(echo_path);
    app.at("/unprotected").get(echo_path);

    // Protected
    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/protected").unwrap(),
    );
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), envoy::StatusCode::Unauthorized);

    let mut req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/protected").unwrap(),
    );
    req.insert_header("X-Auth", "secret_key");
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), envoy::StatusCode::Ok);

    // Unprotected
    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/unprotected").unwrap(),
    );
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), envoy::StatusCode::Ok);

    let mut req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/unprotected").unwrap(),
    );
    req.insert_header("X-Auth", "secret_key");
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), envoy::StatusCode::Ok);
}

#[async_std::test]
async fn app_middleware() {
    let mut app = envoy::new();
    app.with(auth_middleware);
    app.at("/foo").get(echo_path);

    // Foo
    let req = http::Request::new(Method::Get, Url::parse("http://localhost/foo").unwrap());
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), envoy::StatusCode::Unauthorized);

    let mut req = http::Request::new(Method::Get, Url::parse("http://localhost/foo").unwrap());
    req.insert_header("X-Auth", "secret_key");
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), envoy::StatusCode::Ok);
}
