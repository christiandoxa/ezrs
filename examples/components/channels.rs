//! Component example: bounded channels and select-style receive.

use ezrs::{Result, channel, select_recv2};

#[ezrs::main]
async fn main() -> Result<()> {
    let (numbers_tx, mut numbers_rx) = channel(8);
    let (words_tx, mut words_rx) = channel(8);

    numbers_tx.send(7).await?;
    words_tx.send(String::from("ready")).await?;

    match select_recv2(&mut numbers_rx, &mut words_rx).await {
        ezrs::Select2::Left(number) => println!("number: {number}"),
        ezrs::Select2::Right(word) => println!("word: {word}"),
        ezrs::Select2::Closed => println!("closed"),
    }

    Ok(())
}
