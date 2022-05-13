use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::time::{interval, sleep};

struct StateKeeper {
    // Must be kept even
    state: u64,

    // count the number of each method call happening
    ticks: AtomicUsize,
    evaluates: AtomicUsize,
}

impl StateKeeper {
    async fn tick(&mut self) {
        self.ticks.fetch_add(1, Ordering::SeqCst);
        // intermediate state change
        self.state += 1;

        // pretend to do some io
        sleep(Duration::from_millis(1100)).await;

        // restore state to
        self.state += 1;
    }

    async fn evaluate(&mut self) {
        self.evaluates.fetch_add(1, Ordering::SeqCst);
        assert!(self.state % 2 == 0);
        println!(
            "evaluating, seen {} ticks and {} evaluates",
            self.ticks.load(Ordering::SeqCst),
            self.evaluates.load(Ordering::SeqCst)
        );
    }
}

async fn core_loop() {
    let mut state = StateKeeper {
        state: 4,
        ticks: AtomicUsize::default(),
        evaluates: AtomicUsize::default(),
    };

    let mut progress_interval = interval(Duration::from_millis(5000));

    {
        // clear the first immediate interval
        progress_interval.tick().await;
    }

    loop {
        let scope = moro::async_scope!(|scope| {
            let tick = scope.spawn(async {
                loop {
                    state.tick().await;
                    state.evaluate().await;
                }
            });
            let timer = scope.spawn(async {
                loop {
                    progress_interval.tick().await;
                    println!("making progress!!");
                }
            });

            futures::future::join(tick, timer).await;
        })
        .infallible();

        scope.await;
    }
}

#[tokio::main]
async fn main() {
    core_loop().await;
}
