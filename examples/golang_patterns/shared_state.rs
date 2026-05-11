//! Go pattern: shared mutable app state protected by a mutex.

use std::collections::HashMap;

use ezrs::{App, Context, Result, SharedMut};

#[derive(Clone)]
struct State {
    cache: SharedMut<HashMap<String, String>>,
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new()
        .state(State {
            cache: SharedMut::new(HashMap::new()),
        })
        .command(put)
        .run()
        .await
}

async fn put(ctx: Context) -> Result<()> {
    let state = ctx.state::<State>()?;
    state
        .cache
        .update(|cache| {
            cache.insert(String::from("key"), String::from("value"));
        })
        .await;
    ctx.println("stored");
    Ok(())
}
