use ezrs::prelude::*;

async fn hello(ctx: Context) -> Result<()> {
    let name = ctx.arg_or("name", "world");
    ctx.println(format!("hello {name}"));
    Ok(())
}

#[ezrs::test]
async fn facade_app_test_runs_command() {
    let res = App::new()
        .command("hello", hello)
        .test()
        .args(["hello", "--name", "Ayu"])
        .run()
        .await;

    res.assert_success();
    res.assert_stdout_contains("Ayu");
}

#[ezrs::test]
async fn shared_mut_works_from_facade() {
    let value = SharedMut::new(1_u64);
    value.update(|n| *n += 1).await;
    assert_eq!(*value.read().await, 2);
}

#[ezrs::main]
async fn macro_runtime_probe() -> Result<()> {
    Ok(())
}

#[test]
fn main_macro_wrapper_runs() {
    macro_runtime_probe().expect("main macro should run");
}
