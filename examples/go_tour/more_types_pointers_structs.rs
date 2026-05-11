//! Go Tour mapping: pointers and structs.
//!
//! Rust references borrow values. Mutation through references requires `&mut`.

use ezrs::{App, Context, Result};

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn move_right(point: &mut Point) {
    point.x += 1;
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(point).run().await
}

async fn point(ctx: Context) -> Result<()> {
    let mut point = Point { x: 1, y: 2 };
    move_right(&mut point);
    ctx.println(format!("{},{}", point.x, point.y));
    Ok(())
}
