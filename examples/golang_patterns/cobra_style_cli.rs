//! Go pattern: cobra command with pflag-style schema.

use ezrs::{App, ArgSpec, CommandSpec, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    let scan = CommandSpec::new()
        .arg(
            ArgSpec::option("path")
                .short('p')
                .required()
                .help("Path to scan"),
        )
        .arg(ArgSpec::flag("recursive").short('r'))
        .arg(ArgSpec::option("limit").default("100").env("SCAN_LIMIT"));

    App::new()
        .name("cobra-style")
        .command_with(run, scan)
        .run()
        .await
}

async fn run(ctx: Context) -> Result<()> {
    ctx.println(format!(
        "path={} recursive={} limit={}",
        ctx.arg("path")?,
        ctx.flag("recursive"),
        ctx.arg("limit")?
    ));
    Ok(())
}
