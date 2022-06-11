use http_types::{self, Method, Url};

#[async_std::test]
async fn test_missing_param() -> envoy::Result {
    async fn greet(ctx: &mut envoy::Context) -> envoy::Result {
        assert_eq!(ctx.param("name")?, "Param \"name\" not found");
        Ok(ctx.res.set_status(http_types::StatusCode::Ok))
    }

    let mut server = envoy::new();
    server.at("/").get(greet);

    let ctx = http_types::Request::new(Method::Get, Url::parse("http://example.com/")?);
    let res: http_types::Response = server.respond(ctx).await?;
    assert_eq!(res.status(), 500);
    Ok(())
}

#[async_std::test]
async fn hello_world_parametrized() -> envoy::Result {
    async fn greet(ctx: &mut envoy::Context) -> envoy::Result {
        let body = format!("{} says hello", ctx.param("name").unwrap_or("nori"));
        Ok(ctx.res.set_body(body))
    }

    let mut server = envoy::new();
    server.at("/").get(greet);
    server.at("/:name").get(greet);

    let req = http_types::Request::new(Method::Get, Url::parse("http://example.com/")?);
    let mut res: http_types::Response = server.respond(req).await?;
    assert_eq!(res.body_string().await?, "nori says hello");

    let req = http_types::Request::new(Method::Get, Url::parse("http://example.com/iron")?);
    let mut res: http_types::Response = server.respond(req).await?;
    assert_eq!(res.body_string().await?, "iron says hello");
    Ok(())
}
