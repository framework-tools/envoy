use envoy::Body;

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    envoy::log::start();
    let mut app = envoy::new();
    app.at("/").get(|_| async {
        // File sends are chunked by default.
        Ok(Body::from_file(file!()).await?)
    });
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
