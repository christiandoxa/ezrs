//! Go pattern: sentinel and custom errors become typed constructors.

use ezrs::{App, Context, Error, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(find).run().await
}

async fn find(ctx: Context) -> Result<()> {
    let key = ctx.arg_or("key", "missing");
    if key == "missing" {
        return Err(Error::not_found("key"));
    }
    Err(Error::msg(format!("load key {key}: backend unavailable")))
}
