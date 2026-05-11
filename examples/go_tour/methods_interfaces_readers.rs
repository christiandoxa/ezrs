//! Go Tour mapping: readers.
//!
//! Rust has standard `std::io::Read` and Tokio async read traits.

use std::io::Read;

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(read).run().await
}

async fn read(ctx: Context) -> Result<()> {
    let mut cursor = std::io::Cursor::new(b"hello".to_vec());
    let mut text = String::new();
    cursor.read_to_string(&mut text)?;
    ctx.println(text);
    Ok(())
}
