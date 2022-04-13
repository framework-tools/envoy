use http_types::{self, Method, Url};
use envoy::{self, Response, Result, Context};

#[async_std::test]
async fn test_missing_param() -> envoy::Result<()> {
    async fn greet(ctx: Context<()>) -> Result<Response> {
        assert_eq!(ctx.param("name")?, "Param \"name\" not found");
        Ok(Response::new(200))
    }

    let mut server = envoy::new();
    server.at("/").get(greet);

    let ctx = http_types::Request::new(Method::Get, Url::parse("http://example.com/")?);
    let res: http_types::Response = server.respond(ctx).await?;
    assert_eq!(res.status(), 500);
    Ok(())
}

#[async_std::test]
async fn hello_world_parametrized() -> Result<()> {
    async fn greet(ctx: envoy::Context<()>) -> Result<impl Into<Response>> {
        let body = format!("{} says hello", ctx.param("name").unwrap_or("nori"));
        Ok(Response::builder(200).body(body))
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
