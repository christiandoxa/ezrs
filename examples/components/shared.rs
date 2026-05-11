//! Go pattern: shared read-only dependency.

use ezrs::{App, Context, Result, Shared};

#[derive(Clone)]
struct State {
    greeting: Shared<String>,
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new()
        .state(State {
            greeting: Shared::new(String::from("hello")),
        })
        .command("hello", hello)
        .run()
        .await
}

async fn hello(ctx: Context) -> Result<()> {
    let state = ctx.state::<State>()?;
    ctx.println(format!("{} world", state.greeting.get()));
    Ok(())
}
