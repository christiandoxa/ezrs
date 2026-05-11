//! Go pattern: os.WriteFile plus temp-file rename, lock files, and typed state files.

use ezrs::{App, Context, Result};

#[derive(serde::Deserialize, serde::Serialize)]
struct CounterState {
    name: String,
    runs: u64,
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(save).run().await
}

async fn save(ctx: Context) -> Result<()> {
    let state_path = ctx.arg_or("state", "var/state.json");
    let lock_path = format!("{state_path}.lock");
    let _lock = ctx.fs().try_lock(&lock_path)?;

    let mut state = if ctx.fs().exists(&state_path) {
        ctx.fs().read_json::<CounterState>(&state_path).await?
    } else {
        CounterState {
            name: "local-worker".to_string(),
            runs: 0,
        }
    };

    state.runs += 1;
    ctx.fs().write_json(&state_path, &state).await?;
    ctx.fs()
        .atomic_write_string("var/last-run.txt", format!("{} {}\n", state.name, state.runs))
        .await?;

    ctx.println(format!("saved run {}", state.runs));
    Ok(())
}
