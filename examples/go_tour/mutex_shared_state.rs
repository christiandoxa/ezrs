//! Go Tour mapping: sync.Mutex.
//!
//! ezrs uses `SharedMut<T>` for common async shared mutable app state.

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
        .command(count)
        .run()
        .await
}

async fn count(ctx: Context) -> Result<()> {
    let state = ctx.state::<State>()?;

    for _ in 0..5 {
        let counter = state.counter.clone();
        ctx.spawn(async move {
            counter.update(|value| *value += 1).await;
            Ok(())
        });
    }

    ctx.join_all().await?;
    ctx.println(format!("count={}", *state.counter.read().await));
    Ok(())
}
