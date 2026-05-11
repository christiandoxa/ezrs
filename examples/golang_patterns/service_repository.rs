//! Go pattern: command -> service -> repository layering.

use ezrs::{App, Context, Result};

trait Repository: Clone + Send + Sync + 'static {
    fn find_name(&self, id: u64) -> Result<String>;
}

#[derive(Clone)]
struct MemoryRepository;

impl Repository for MemoryRepository {
    fn find_name(&self, id: u64) -> Result<String> {
        Ok(format!("user-{id}"))
    }
}

#[derive(Clone)]
struct Service<R: Repository> {
    repo: R,
}

impl<R: Repository> Service<R> {
    fn name(&self, id: u64) -> Result<String> {
        self.repo.find_name(id)
    }
}

#[derive(Clone)]
struct State {
    service: Service<MemoryRepository>,
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new()
        .state(State {
            service: Service {
                repo: MemoryRepository,
            },
        })
        .command(user)
        .run()
        .await
}

async fn user(ctx: Context) -> Result<()> {
    let state = ctx.state::<State>()?;
    ctx.println(state.service.name(42)?);
    Ok(())
}
