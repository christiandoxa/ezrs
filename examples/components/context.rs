//! Go pattern: context.Context plus an app capability handle.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("inspect", inspect).run().await
}

async fn inspect(ctx: Context) -> Result<()> {
    let path = ctx.arg_or("path", ".");
    let name = ctx.arg("name").unwrap_or_else(|_| String::from("world"));
    let verbose = ctx.flag("verbose");
    let home = ctx.env("HOME")?;

    ctx.log().info(format!("inspecting {path}"));
    ctx.println(format!("hello {name}; verbose={verbose}; home={home}"));
    ctx.eprintln("diagnostic output goes to stderr");
    ctx.println(format!("current dir exists: {}", ctx.fs().exists(".")));
    ctx.check_cancelled()?;
    Ok(())
}
