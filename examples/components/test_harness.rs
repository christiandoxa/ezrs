//! Go pattern: handler tests with fake input and captured output.

use ezrs::{App, Context, Result};
#[cfg(test)]
use ezrs::EnvMap;

async fn hello(ctx: Context) -> Result<()> {
    ctx.println(format!("hello {}", ctx.arg_or("name", "world")));
    Ok(())
}

async fn fail(_: Context) -> Result<()> {
    Err(ezrs::Error::msg("boom"))
}

#[ezrs::test]
async fn hello_works() {
    let res = App::new()
        .command(hello)
        .test()
        .args(["hello", "--name", "Ayu"])
        .run()
        .await;

    res.assert_success();
    res.assert_stdout_eq("hello Ayu\n");
}

#[ezrs::test]
async fn env_is_injected_without_global_mutation() {
    async fn read_env(ctx: Context) -> Result<()> {
        ctx.println(ctx.env("APP_NAME")?);
        Ok(())
    }

    let res = App::new()
        .command(read_env)
        .test()
        .env(EnvMap::new().set("APP_NAME", "demo"))
        .args(["read_env"])
        .run()
        .await;

    res.assert_success();
    res.assert_stdout_eq("demo\n");
}

#[ezrs::test]
async fn failure_is_captured() {
    let res = App::new()
        .command(fail)
        .test()
        .args(["fail"])
        .run()
        .await;

    res.assert_failure();
    res.assert_stderr_contains("boom");
}

fn main() {
    let _app = App::new().command(hello).command(fail);
}
