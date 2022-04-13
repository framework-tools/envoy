use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[derive(Clone)]
struct State {
    value: Arc<AtomicU32>,
}

impl State {
    fn new() -> Self {
        Self {
            value: Arc::new(AtomicU32::new(0)),
        }
    }
}

#[async_std::main]
async fn main() -> envoy::Result<()> {
    envoy::log::start();
    let mut app = envoy::with_state(State::new());
    app.at("/").get(|ctx: envoy::Context<State>| async move {
        let state = ctx.state();
        let value = state.value.load(Ordering::Relaxed);
        Ok(format!("{}\n", value))
    });
    app.at("/inc").get(|ctx: envoy::Context<State>| async move {
        let state = ctx.state();
        let value = state.value.fetch_add(1, Ordering::Relaxed) + 1;
        Ok(format!("{}\n", value))
    });
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
