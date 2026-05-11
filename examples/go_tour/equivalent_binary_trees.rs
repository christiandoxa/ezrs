//! Go Tour mapping: equivalent binary trees exercise.
//!
//! Rust can model tree walks with iterators, vectors, or channels. This small version uses values.

use ezrs::{App, Context, Result};

#[derive(Debug)]
struct Tree {
    value: i32,
    left: Option<Box<Tree>>,
    right: Option<Box<Tree>>,
}

impl Tree {
    fn leaf(value: i32) -> Self {
        Self {
            value,
            left: None,
            right: None,
        }
    }

    fn node(value: i32, left: Tree, right: Tree) -> Self {
        Self {
            value,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }

    fn walk(&self, values: &mut Vec<i32>) {
        if let Some(left) = &self.left {
            left.walk(values);
        }
        values.push(self.value);
        if let Some(right) = &self.right {
            right.walk(values);
        }
    }
}

fn same(left: &Tree, right: &Tree) -> bool {
    let mut left_values = Vec::new();
    let mut right_values = Vec::new();
    left.walk(&mut left_values);
    right.walk(&mut right_values);
    left_values == right_values
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(trees).run().await
}

async fn trees(ctx: Context) -> Result<()> {
    let first = Tree::node(2, Tree::leaf(1), Tree::leaf(3));
    let second = Tree::node(2, Tree::leaf(1), Tree::leaf(3));
    ctx.println(format!("same={}", same(&first, &second)));
    Ok(())
}
