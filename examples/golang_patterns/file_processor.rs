//! Go pattern: small file processor CLI.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(process).run().await
}

async fn process(ctx: Context) -> Result<()> {
    let input = ctx.arg_or("input", "input.txt");
    let output = ctx.arg_or("output", "output.txt");
    let text = ctx.fs().read_to_string(input).await?;
    ctx.fs().write_string(output, text.to_uppercase()).await?;
    ctx.println("processed");
    Ok(())
}
