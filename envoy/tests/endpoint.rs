use envoy::http::{Method, Response, Request, Url};

#[tokio::test]
async fn should_accept_boxed_endpoints() {
    fn endpoint() -> Box<dyn envoy::Endpoint> {
        async fn inner(ctx: &mut envoy::Context) -> envoy::Result {
            ctx.set_body("hello world");
            Ok(())
        }
        Box::new(inner)
    }

    let mut app = envoy::Server::new();
    app.at("/").get(endpoint());

    let mut response: Response = app
        .respond(Request::new(
            Method::Get,
            Url::parse("http://example.com/").unwrap(),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.take_body().into_string().await.unwrap(),
        "hello world"
    );
}
