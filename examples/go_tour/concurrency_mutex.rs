//! Go Tour mapping: sync.Mutex.
//!
//! ezrs exposes `SharedMut<T>` for common app state.

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
        .command(mutex)
        .run()
        .await
}

async fn mutex(ctx: Context) -> Result<()> {
    let state = ctx.state::<State>()?;
    state.counter.update(|value| *value += 1).await;
    ctx.println(*state.counter.read().await);
    Ok(())
}
