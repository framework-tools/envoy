mod test_utils;
use hyper::Response;
use test_utils::ServerTestingExt;
use envoy::{Error, StatusCode, Context, Endpoint};

struct StringEndpoint(String);
#[async_trait::async_trait]
impl Endpoint for StringEndpoint {
    async fn call(&self, ctx: &mut envoy::Context) -> Result<crate::Response<crate::Body>, Error> {
        Ok(self.0.clone().into())
    }
}

async fn add_one(ctx: &mut Context) -> envoy::Result {
    let num: i64 = ctx
        .param("num")?
        .parse()
        .map_err(|err| Error::new(StatusCode::BAD_REQUEST, err))?;
    Ok(ctx.res.set_body((num + 1).to_string()))
}

async fn add_two(ctx: &mut Context) -> envoy::Result {
    let one: i64 = ctx
        .param("one")?
        .parse()
        .map_err(|err| Error::new(StatusCode::BAD_REQUEST, err))?;
    let two: i64 = ctx
        .param("two")?
        .parse()
        .map_err(|err| Error::new(StatusCode::BAD_REQUEST, err))?;
    Ok(ctx.res.set_body((one + two).to_string()))
}

async fn echo_param(ctx: &mut envoy::Context) -> envoy::Result {
    match ctx.param("param").map(|param| param.to_string()) {
        Ok(path) => Ok(ctx.res.set_body(path)),
        Err(_) => Ok(ctx.res.set_status(StatusCode::NOT_FOUND)),
    }
}

async fn echo_wildcard(ctx: &mut Context) -> envoy::Result {
    match ctx.wildcard().map(|param| param.to_string()) {
        Some(path) => Ok(ctx.set_body(path)),
        None => Ok(ctx.res.set_status(StatusCode::NOT_FOUND)),
    }
}

#[tokio::test]
async fn param() -> envoy::Result {
    let mut app = envoy::Server::new();
    app.at("/add_one/:num").get(add_one);
    assert_eq!(app.get("/add_one/3").recv_string().await?, "4");
    assert_eq!(app.get("/add_one/-7").recv_string().await?, "-6");
    Ok(())
}

#[tokio::test]
async fn invalid_segment_error() -> envoy::Result {
    let mut app = envoy::new();
    app.at("/add_one/:num").get(add_one);
    assert_eq!(
        app.get("/add_one/a").await?.status(),
        StatusCode::BAD_REQUEST
    );
    Ok(())
}

#[tokio::test]
async fn not_found_error() -> envoy::Result {
    let mut app = envoy::new();
    app.at("/add_one/:num").get(add_one);
    assert_eq!(app.get("/add_one/").await?.status(), StatusCode::NOT_FOUND);
    Ok(())
}

#[tokio::test]
async fn wildcard() -> envoy::Result {
    let mut app = envoy::new();
    app.at("/echo/*").get(echo_wildcard);
    assert_eq!(app.get("/echo/some_path").recv_string().await?, "some_path");
    assert_eq!(
        app.get("/echo/multi/segment/path").recv_string().await?,
        "multi/segment/path"
    );
    assert_eq!(app.get("/echo/").await?.status(), StatusCode::OK);
    assert_eq!(app.get("/echo").await?.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn multi_param() -> envoy::Result {
    let mut app = envoy::new();
    app.at("/add_two/:one/:two/").get(add_two);
    assert_eq!(app.get("/add_two/1/2/").recv_string().await?, "3");
    assert_eq!(app.get("/add_two/-1/2/").recv_string().await?, "1");
    assert_eq!(app.get("/add_two/1").await?.status(), StatusCode::NOT_FOUND);
    Ok(())
}

#[tokio::test]
async fn wildcard_last_segment() -> envoy::Result {
    let mut app = envoy::new();
    app.at("/echo/:param/*").get(echo_param);
    assert_eq!(app.get("/echo/one/two").recv_string().await?, "one");
    assert_eq!(
        app.get("/echo/one/two/three/four").recv_string().await?,
        "one"
    );
    Ok(())
}

#[tokio::test]
async fn ambiguous_router_wildcard_vs_star() -> envoy::Result {
    let mut app = envoy::new();

    app.at("/:one/:two").get(StringEndpoint("one/two".to_string()));
    app.at("/posts/*").get(StringEndpoint("posts/*".to_string()));
    assert_eq!(app.get("/posts/10").recv_string().await?, "posts/*");
    Ok(())
}
