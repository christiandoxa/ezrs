//! Go pattern: fake implementation for handler tests.

use ezrs::{App, Context, Result};

trait Store: Clone + Send + Sync + 'static {
    fn get(&self) -> String;
}

#[derive(Clone)]
struct FakeStore;

impl Store for FakeStore {
    fn get(&self) -> String {
        String::from("fake")
    }
}

#[derive(Clone)]
struct State<S: Store> {
    store: S,
}

async fn get(ctx: Context) -> Result<()> {
    let state = ctx.state::<State<FakeStore>>()?;
    ctx.println(state.store.get());
    Ok(())
}

#[ezrs::test]
async fn handler_uses_fake_store() {
    let res = App::new()
        .state(State { store: FakeStore })
        .command(get)
        .test()
        .args(["get"])
        .run()
        .await;

    res.assert_success();
    res.assert_stdout_contains("fake");
}

fn main() {
    let _app = App::new()
        .state(State { store: FakeStore })
        .command(get);
}
