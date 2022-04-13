#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    envoy::log::start();
    let mut app = envoy::new();

    app.with(envoy::sessions::SessionMiddleware::new(
        envoy::sessions::MemoryStore::new(),
        std::env::var("TIDE_SECRET")
            .expect(
                "Please provide a TIDE_SECRET value of at \
                    least 32 bytes in order to run this example",
            )
            .as_bytes(),
    ));

    app.with(envoy::utils::Before(
        |mut ctx: envoy::Context<()>| async move {
            let session = ctx.session_mut();
            let visits: usize = session.get("visits").unwrap_or_default();
            session.insert("visits", visits + 1).unwrap();
            ctx
        },
    ));

    app.at("/").get(|ctx: envoy::Context<()>| async move {
        let visits: usize = ctx.session().get("visits").unwrap();
        Ok(format!("you have visited this website {} times", visits))
    });

    app.at("/reset")
        .get(|mut ctx: envoy::Context<()>| async move {
            ctx.session_mut().destroy();
            Ok(envoy::Redirect::new("/"))
        });

    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
