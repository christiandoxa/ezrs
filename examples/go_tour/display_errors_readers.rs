//! Go Tour mapping: Stringer, errors, readers, and images.
//!
//! Rust uses `Display`, `Error`, `Read`, and small traits for image-like abstractions.

use std::{
    fmt::{self, Display},
    io::{Cursor, Read},
};

use ezrs::{App, Context, Error, Result};

struct User {
    name: String,
}

impl Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "user:{}", self.name)
    }
}

trait ImageLike {
    fn bounds(&self) -> (usize, usize);
    fn pixel(&self, x: usize, y: usize) -> u8;
}

struct GrayImage {
    width: usize,
    height: usize,
}

impl ImageLike for GrayImage {
    fn bounds(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn pixel(&self, x: usize, y: usize) -> u8 {
        ((x + y) % 255) as u8
    }
}

fn read_word(mut reader: impl Read) -> Result<String> {
    let mut text = String::new();
    reader.read_to_string(&mut text)?;
    text.split_whitespace()
        .next()
        .map(ToOwned::to_owned)
        .ok_or_else(|| Error::not_found("word"))
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("display", display).run().await
}

async fn display(ctx: Context) -> Result<()> {
    let user = User {
        name: ctx.arg_or("name", "Ada"),
    };
    let word = read_word(Cursor::new("hello reader"))?;
    let image = GrayImage {
        width: 2,
        height: 2,
    };
    let (width, height) = image.bounds();

    ctx.println(format!(
        "{user} word={word} image={width}x{height} p={}",
        image.pixel(1, 1)
    ));
    Ok(())
}
