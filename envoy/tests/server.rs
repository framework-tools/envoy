mod test_utils;
use tokio::task;
use envoy::Endpoint;
use std::time::Duration;

use envoy::Body;
use serde::{Deserialize, Serialize};

#[tokio::test]
async fn hello_world() -> envoy::Result {
    struct PortEndpoint(u16);

    #[async_trait::async_trait]
    impl Endpoint for PortEndpoint {
        async fn call(&self, ctx: &mut envoy::Context) -> envoy::Result {
            assert_eq!(ctx.body_string().await.unwrap(), "nori".to_string());
            assert!(ctx.local_addr().unwrap().contains(&self.0.to_string()));
            assert!(ctx.peer_addr().is_some());
            Ok(ctx.set_body("says hello"))
        }
    }


    let port = test_utils::find_port().await;
    let server = task::spawn(async move {
        let mut app = envoy::new();
        app.at("/").get(PortEndpoint(port));
        app.listen(("localhost", port)).await?;
        Result::<(), http_types::Error>::Ok(())
    });

    let client = task::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let string = surf::get(format!("http://localhost:{}", port))
            .body(Body::from_string("nori".to_string()))
            .recv_string()
            .await
            .unwrap();
        assert_eq!(string, "says hello");
        Ok(())
    });

    server.race(client).await
}

#[tokio::test]
async fn echo_server() -> envoy::Result {
    async fn echo(ctx: &mut envoy::Context) -> envoy::Result {
        Ok(ctx.res.set_body(ctx.req.body_string().await?))
    }

    let port = test_utils::find_port().await;
    let server = task::spawn(async move {
        let mut app = envoy::new();
        app.at("/").get(echo);

        app.listen(("localhost", port)).await?;
        Result::<(), http_types::Error>::Ok(())
    });

    let client = task::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let string = surf::get(format!("http://localhost:{}", port))
            .body(Body::from_string("chashu".to_string()))
            .recv_string()
            .await
            .unwrap();
        assert_eq!(string, "chashu".to_string());
        Ok(())
    });

    server.race(client).await
}

#[tokio::test]
async fn json() -> envoy::Result {
    #[derive(Deserialize, Serialize)]
    struct Counter {
        count: usize,
    }

    async fn increment_counter(ctx: &mut envoy::Context) -> envoy::Result {
        let mut counter: Counter = ctx.body_json().await.unwrap();
        assert_eq!(counter.count, 0);
        counter.count = 1;
        Ok(ctx.res.set_body(Body::from_json(&counter)?))
    }

    let port = test_utils::find_port().await;
    let server = task::spawn(async move {
        let mut app = envoy::new();
        app.at("/").get(increment_counter);
        app.listen(("localhost", port)).await?;
        Result::<(), http_types::Error>::Ok(())
    });

    let client = task::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let counter: Counter = surf::get(format!("http://localhost:{}", &port))
            .body(Body::from_json(&Counter { count: 0 })?)
            .recv_json()
            .await
            .unwrap();
        assert_eq!(counter.count, 1);
        Ok(())
    });

    server.race(client).await
}
