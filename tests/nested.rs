mod test_utils;
use test_utils::ServerTestingExt;

#[async_std::test]
async fn nested() -> envoy::Result {
    let mut inner = envoy::new();
    inner.at("/foo").get(|_| async { Ok("foo") });
    inner.at("/bar").get(|_| async { Ok("bar") });

    let mut outer = envoy::new();
    // Nest the inner app on /foo
    outer.at("/foo").nest(inner);

    assert_eq!(outer.get("/foo/foo").recv_string().await?, "foo");
    assert_eq!(outer.get("/foo/bar").recv_string().await?, "bar");
    Ok(())
}

#[async_std::test]
async fn nested_middleware() -> envoy::Result {
    let echo_path = |ctx: envoy::Context| async move { Ok(ctx.url().path().to_string()) };
    let mut app = envoy::new();
    let mut inner_app = envoy::new();
    inner_app.with(envoy::utils::After(|mut res: envoy::Response| async move {
        res.insert_header("x-envoy-test", "1");
        Ok(res)
    }));
    inner_app.at("/echo").get(echo_path);
    inner_app.at("/:foo/bar").strip_prefix().get(echo_path);
    app.at("/foo").nest(inner_app);
    app.at("/bar").get(echo_path);

    let mut res = app.get("/foo/echo").await?;
    assert_eq!(res["X-Envoy-Test"], "1");
    assert_eq!(res.status(), 200);
    assert_eq!(res.body_string().await?, "/echo");

    let mut res = app.get("/foo/x/bar").await?;
    assert_eq!(res["X-Envoy-Test"], "1");
    assert_eq!(res.status(), 200);
    assert_eq!(res.body_string().await?, "/");

    let mut res = app.get("/bar").await?;
    assert!(res.header("X-Envoy-Test").is_none());
    assert_eq!(res.status(), 200);
    assert_eq!(res.body_string().await?, "/bar");
    Ok(())
}

#[async_std::test]
async fn nested_with_different_state() -> envoy::Result {
    let mut outer = envoy::new();
    let mut inner = envoy::with_state(42);
    inner.at("/").get(|req: envoy::Context<i32>| async move {
        let num = req.state();
        Ok(format!("the number is {}", num))
    });
    outer.at("/").get(|_| async { Ok("Hello, world!") });
    outer.at("/foo").nest(inner);

    assert_eq!(outer.get("/foo").recv_string().await?, "the number is 42");
    assert_eq!(outer.get("/").recv_string().await?, "Hello, world!");
    Ok(())
}
