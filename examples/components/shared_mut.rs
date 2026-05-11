//! Go pattern: sync.Mutex-protected shared state.

use ezrs::{App, Context, Result, SharedMut};

#[derive(Clone)]
struct State {
    counter: SharedMut<u64>,
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new()
        .state(State {
            counter: SharedMut::new(0),
        })
        .command("count", count)
        .run()
        .await
}

async fn count(ctx: Context) -> Result<()> {
    let state = ctx.state::<State>()?;
    state.counter.update(|n| *n += 1).await;
    {
        let mut value = state.counter.write().await;
        *value += 1;
    }
    let n = *state.counter.read().await;
    ctx.println(format!("count: {n}"));
    Ok(())
}
