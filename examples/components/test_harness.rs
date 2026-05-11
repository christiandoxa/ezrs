//! Go pattern: handler tests with fake input and captured output.

use ezrs::{App, Context, Result};

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
    res.assert_stdout_contains("Ayu");
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
