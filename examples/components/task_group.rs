//! Go pattern: goroutine group plus WaitGroup-style join.

use ezrs::{Result, TaskGroup};

#[ezrs::main]
async fn main() -> Result<()> {
    let group = TaskGroup::new().cancel_on_error(true);
    let cancellation = group.cancellation();

    for item in ["alpha", "beta", "gamma"] {
        let cancellation = cancellation.clone();
        group.spawn(async move {
            cancellation.check_cancelled()?;
            println!("processed {item}");
            Ok(())
        });
    }

    group.join().await
}
