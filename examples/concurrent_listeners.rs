use envoy::{Context};

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    envoy::log::start();
    let mut app = envoy::new();

    app.at("/").get(|request: Context<_>| async move {
        Ok(format!(
            "Hi! You reached this app through: {}",
            request.local_addr().unwrap_or("an unknown port")
        ))
    });

    app.listen(vec!["localhost:8000", "localhost:8081"]).await?;

    Ok(())
}
