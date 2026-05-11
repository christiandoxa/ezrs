//! Go Tour mapping: type parameters and generic types.

use ezrs::{App, Context, Result};

fn index_of<T: PartialEq>(items: &[T], target: &T) -> Option<usize> {
    items.iter().position(|item| item == target)
}

struct Stack<T> {
    values: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Self { values: Vec::new() }
    }

    fn push(&mut self, value: T) {
        self.values.push(value);
    }

    fn pop(&mut self) -> Option<T> {
        self.values.pop()
    }
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(generic).run().await
}

async fn generic(ctx: Context) -> Result<()> {
    let values = ["go", "rust", "ezrs"];
    let index = index_of(&values, &"rust").unwrap_or_default();

    let mut stack = Stack::new();
    stack.push(String::from("first"));
    stack.push(String::from("second"));

    ctx.println(format!(
        "index={index} top={}",
        stack.pop().unwrap_or_default()
    ));
    Ok(())
}
