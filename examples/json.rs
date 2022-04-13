use serde::{Deserialize, Serialize};
use envoy::{prelude::*, Context}; // Pulls in the json! macro.
use envoy::{Body};

#[derive(Deserialize, Serialize)]
struct Cat {
    name: String,
}

#[async_std::main]
async fn main() -> envoy::Result<()> {
    envoy::log::start();
    let mut app = envoy::new();

    app.at("/submit").post(|mut ctx: Context<()>| async move {
        let cat: Cat = ctx.body_json().await?;
        println!("cat name: {}", cat.name);

        let cat = Cat {
            name: "chashu".into(),
        };

        Body::from_json(&cat)
    });

    app.at("/animals").get(|_| async {
        Ok(json!({
            "meta": { "count": 2 },
            "animals": [
                { "type": "cat", "name": "chashu" },
                { "type": "cat", "name": "nori" }
            ]
        }))
    });

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
