//! Go Tour mapping: pointers, structs, struct fields, pointer-to-struct behavior, literals.
//!
//! Rust uses references, owned structs, mutable borrows, and `Box<T>` for heap ownership.

use ezrs::{App, Context, Result};

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn translate(point: &mut Point, dx: i32, dy: i32) {
    point.x += dx;
    point.y += dy;
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(point).run().await
}

async fn point(ctx: Context) -> Result<()> {
    let mut point = Point { x: 1, y: 2 };
    translate(&mut point, 3, 4);

    let boxed = Box::new(point);
    ctx.println(format!("point={},{}", boxed.x, boxed.y));
    Ok(())
}
