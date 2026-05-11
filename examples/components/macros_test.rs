//! Go pattern: handler-style async tests.

use ezrs::{App, Context, Result};

async fn hello(ctx: Context) -> Result<()> {
    ctx.println("hello test");
    Ok(())
}

#[ezrs::test]
async fn command_test_works() {
    let res = App::new()
        .command("hello", hello)
        .test()
        .args(["hello"])
        .run()
        .await;

    res.assert_success();
    res.assert_stdout_contains("hello test");
}

fn main() {
    let _app = App::new().command("hello", hello);
}
