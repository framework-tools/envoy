use envoy::http::Cookie;
use envoy::{Response, StatusCode, Context};

/// Envoy will use the the `Cookies`'s `Extract` implementation to build this parameter.
///
async fn retrieve_cookie(ctx: Context<()>) -> envoy::Result<String> {
    Ok(format!("hello cookies: {:?}", ctx.cookie("hello").unwrap()))
}

async fn insert_cookie(_ctx: Context<()>) -> envoy::Result {
    let mut res = Response::new(StatusCode::Ok);
    res.insert_cookie(Cookie::new("hello", "world"));
    Ok(res)
}

async fn remove_cookie(_ctx: Context<()>) -> envoy::Result {
    let mut res = Response::new(StatusCode::Ok);
    res.remove_cookie(Cookie::named("hello"));
    Ok(res)
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    envoy::log::start();
    let mut app = envoy::new();

    app.at("/").get(retrieve_cookie);
    app.at("/set").get(insert_cookie);
    app.at("/remove").get(remove_cookie);
    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
