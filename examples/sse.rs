use envoy::sse;

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    envoy::log::start();
    let mut app = envoy::new();
    app.at("/sse").get(sse::endpoint(|_req, sender| async move {
        sender.send("fruit", "banana", None).await?;
        sender.send("fruit", "apple", None).await?;
        Ok(())
    }));
    app.listen("localhost:8080").await?;
    Ok(())
}
