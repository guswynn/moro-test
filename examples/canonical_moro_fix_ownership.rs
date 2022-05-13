use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::time::{interval, sleep};

#[derive(Debug)]
struct StateKeeper {
    // Must be kept even
    state: u64,

    // count the number of each method call happening
    ticks: AtomicUsize,
    evaluates: AtomicUsize,
}

impl StateKeeper {
    async fn tick(mut self) -> Self {
        self.ticks.fetch_add(1, Ordering::SeqCst);

        // intermediate state change
        self.state += 1;

        // pretend to do some io
        sleep(Duration::from_millis(1100)).await;

        // restore state to
        self.state += 1;

        self
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
    let state = StateKeeper {
        state: 4,
        ticks: AtomicUsize::default(),
        evaluates: AtomicUsize::default(),
    };

    let mut progress_interval = interval(Duration::from_millis(5000));

    {
        // clear the first immediate interval
        progress_interval.tick().await;
    }

    let scope = moro::async_scope!(|scope| {
        let mut tick_join_handle = Box::pin(scope.spawn(state.tick()));
        loop {
            tokio::select! {
                mut state = &mut tick_join_handle=> {
                    state.evaluate().await;
                    tick_join_handle = Box::pin(scope.spawn(state.tick()));
                },
                // As moro is currently implemented, `spawn`-ing here
                // is wrong, every time we drop the "JoinHandle"
                // from the above `state.tick()` winning the race,
                // the interval `tx.send` is lost.
                //
                // Perhaps `JoinHandle`'s for moro should not
                // detach-on-drop.
                //
                // Additionally, something about the inferred scope lifetime here
                // makes it so you actually are unable to scope.spawn this future...
                // ill have to look into this
                _ = progress_interval.tick() => {
                    println!("making progress!!");
                }
            }
        }
    })
    .infallible();

    scope.await;
}

#[tokio::main]
async fn main() {
    core_loop().await;
}
