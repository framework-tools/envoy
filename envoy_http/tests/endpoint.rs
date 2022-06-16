use envoy::{Method, Response, Body};
use hyper::body;

#[tokio::test]
async fn should_accept_boxed_endpoints() {
    fn endpoint() -> Box<dyn envoy::Endpoint> {
        async fn inner(_ctx: &mut envoy::Context) -> envoy::Result {
            Ok(envoy::Response::new("hello world".into()))
        }
        Box::new(inner)
    }

    let mut app = envoy::Server::new();
    app.at("/").get(endpoint());

    let mut response: Response<Body> = app
        .respond(envoy::Request::builder()
            .method(Method::GET)
            .uri("http://example.com/")
            .body(Body::empty())
            .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(
        body::to_bytes(response.body_mut()).await.unwrap(),
        "hello world"
    );
}
