//! Go pattern: error-as-value and explicit propagation.

use ezrs::{App, Context, Error, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("read", read).run().await
}

async fn read(ctx: Context) -> Result<()> {
    let path = ctx.arg("path").map_err(|_| Error::invalid_input("missing --path"))?;
    if path == "missing" {
        return Err(Error::not_found("requested file"));
    }

    let text = std::fs::read_to_string(path)?;
    if text.is_empty() {
        return Err(Error::msg("file is empty"));
    }

    ctx.println(text);
    Ok(())
}
