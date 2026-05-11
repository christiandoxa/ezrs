//! Go pattern: small interfaces become Rust traits.

use ezrs::{App, Context, Result};

trait Store: Send + Sync {
    fn get(&self, key: &str) -> Result<String>;
}

#[derive(Clone)]
struct MemoryStore;

impl Store for MemoryStore {
    fn get(&self, key: &str) -> Result<String> {
        Ok(format!("value for {key}"))
    }
}

#[derive(Clone)]
struct State {
    store: MemoryStore,
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new()
        .state(State { store: MemoryStore })
        .command(get)
        .run()
        .await
}

async fn get(ctx: Context) -> Result<()> {
    let state = ctx.state::<State>()?;
    ctx.println(state.store.get("demo")?);
    Ok(())
}
