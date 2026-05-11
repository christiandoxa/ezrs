//! Go pattern: explicit dependency passing without globals.

use ezrs::{App, Context, Result};

#[derive(Clone)]
struct State {
    app_name: String,
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new()
        .state(State {
            app_name: String::from("demo"),
        })
        .command("hello", hello)
        .run()
        .await
}

async fn hello(ctx: Context) -> Result<()> {
    let state = ctx.state::<State>()?;
    ctx.println(format!("app: {}", state.app_name));
    Ok(())
}
