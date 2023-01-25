use std::time::{Duration, Instant};

use tokio::time;
use tokio_stream::{wrappers::IntervalStream, StreamExt};

#[tokio::main]
async fn main() {
    let cycletime = Duration::from_millis(5);

    let mut interval = tokio::time::interval(cycletime);

    interval.tick().await;

    let task = tokio::task::spawn(async move {
        let mut last_t = tokio::time::Instant::now();
        loop {
            let t = interval.tick().await;

            println!("Hello! {:#?}", t.duration_since(last_t).as_nanos());
            println!("    Elapsed: {:#?}", last_t.elapsed());

            last_t = t;
        }
    });

    task.await.expect("oopie");
}
