mod test_utils;
use async_std::prelude::*;
use async_std::task;
use envoy::Context;
use std::time::Duration;

use envoy::Body;
use serde::{Deserialize, Serialize};

#[test]
fn hello_world() -> envoy::Result<()> {
    task::block_on(async {
        let port = test_utils::find_port().await;
        let server = task::spawn(async move {
            let mut app = envoy::new();
            app.at("/").get(move |mut ctx: Context<()>| async move {
                assert_eq!(ctx.body_string().await.unwrap(), "nori".to_string());
                assert!(ctx.local_addr().unwrap().contains(&port.to_string()));
                assert!(ctx.peer_addr().is_some());
                Ok("says hello")
            });
            app.listen(("localhost", port)).await?;
            Result::<(), http_types::Error>::Ok(())
        });

        let client = task::spawn(async move {
            task::sleep(Duration::from_millis(100)).await;
            let string = surf::get(format!("http://localhost:{}", port))
                .body(Body::from_string("nori".to_string()))
                .recv_string()
                .await
                .unwrap();
            assert_eq!(string, "says hello");
            Ok(())
        });

        server.race(client).await
    })
}

#[test]
fn echo_server() -> envoy::Result<()> {
    task::block_on(async {
        let port = test_utils::find_port().await;
        let server = task::spawn(async move {
            let mut app = envoy::new();
            app.at("/").get(|ctx: Context<()> | async move { Ok(ctx) });

            app.listen(("localhost", port)).await?;
            Result::<(), http_types::Error>::Ok(())
        });

        let client = task::spawn(async move {
            task::sleep(Duration::from_millis(100)).await;
            let string = surf::get(format!("http://localhost:{}", port))
                .body(Body::from_string("chashu".to_string()))
                .recv_string()
                .await
                .unwrap();
            assert_eq!(string, "chashu".to_string());
            Ok(())
        });

        server.race(client).await
    })
}

#[test]
fn json() -> envoy::Result<()> {
    #[derive(Deserialize, Serialize)]
    struct Counter {
        count: usize,
    }

    task::block_on(async {
        let port = test_utils::find_port().await;
        let server = task::spawn(async move {
            let mut app = envoy::new();
            app.at("/").get(|mut ctx: Context<()>| async move {
                let mut counter: Counter = ctx.body_json().await.unwrap();
                assert_eq!(counter.count, 0);
                counter.count = 1;
                Body::from_json(&counter)
            });
            app.listen(("localhost", port)).await?;
            Result::<(), http_types::Error>::Ok(())
        });

        let client = task::spawn(async move {
            task::sleep(Duration::from_millis(100)).await;
            let counter: Counter = surf::get(format!("http://localhost:{}", &port))
                .body(Body::from_json(&Counter { count: 0 })?)
                .recv_json()
                .await
                .unwrap();
            assert_eq!(counter.count, 1);
            Ok(())
        });

        server.race(client).await
    })
}
