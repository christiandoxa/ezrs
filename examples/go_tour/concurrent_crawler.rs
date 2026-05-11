//! Go Tour mapping: web crawler exercise.
//!
//! ezrs v0.1.0 is not a web framework. This example keeps the crawler fake and in-memory.

use std::collections::HashSet;

use ezrs::{App, Context, Result, SharedMut};

#[derive(Clone)]
struct State {
    visited: SharedMut<HashSet<String>>,
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new()
        .state(State {
            visited: SharedMut::new(HashSet::new()),
        })
        .command("crawl", crawl)
        .run()
        .await
}

async fn crawl(ctx: Context) -> Result<()> {
    let state = ctx.state::<State>()?;
    for url in ["/", "/about", "/", "/docs"] {
        let visited = state.visited.clone();
        let output = ctx.clone();
        ctx.spawn(format!("visit-{url}"), async move {
            let inserted = visited.update(|seen| seen.insert(url.to_owned())).await;
            if inserted {
                output.println(format!("visited {url}"));
            }
            Ok(())
        });
    }

    ctx.join_all().await?;
    ctx.println(format!("unique={}", state.visited.read().await.len()));
    Ok(())
}
