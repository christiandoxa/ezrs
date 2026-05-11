//! Go pattern: retry loop with backoff.

use ezrs::{App, Context, Error, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(retry).run().await
}

async fn retry(ctx: Context) -> Result<()> {
    for attempt in 1..=3 {
        match call(attempt) {
            Ok(value) => {
                ctx.println(value);
                return Ok(());
            }
            Err(error) if attempt < 3 => {
                ctx.log().warn(format!("attempt {attempt} failed: {error}"));
                ctx.sleep_secs(1).await;
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn call(attempt: u64) -> Result<&'static str> {
    if attempt < 2 {
        Err(Error::timeout("temporary backend delay"))
    } else {
        Ok("success")
    }
}
