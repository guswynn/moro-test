use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::{interval, sleep};

struct StateKeeper {
    // Must be kept even
    state: Arc<Mutex<u64>>,

    // count the number of each method call happening
    ticks: AtomicUsize,
    evaluates: AtomicUsize,
}

impl StateKeeper {
    async fn tick(&self) {
        self.ticks.fetch_add(1, Ordering::SeqCst);
        let mut state = self.state.lock().await;

        // intermediate state change
        *state += 1;

        // pretend to do some io
        sleep(Duration::from_millis(1100)).await;

        // restore state to
        *state += 1;
    }

    async fn evaluate(&self) {
        self.evaluates.fetch_add(1, Ordering::SeqCst);
        let state = self.state.lock().await;

        assert!(*state % 2 == 0);
        println!(
            "evaluating, seen {} ticks and {} evaluates",
            self.ticks.load(Ordering::SeqCst),
            self.evaluates.load(Ordering::SeqCst)
        );
    }
}

async fn core_loop() {
    let state = StateKeeper {
        state: Arc::new(Mutex::new(4)),
        ticks: AtomicUsize::default(),
        evaluates: AtomicUsize::default(),
    };

    let mut progress_interval = interval(Duration::from_millis(2000));

    {
        // clear the first immediate interval
        progress_interval.tick().await;
    }

    let scope = moro::async_scope!(|scope| {

        // boxing is because currently `moro` "JoinHandle"'s are not `Unpin`,
        // but we can easily imagine they are.
        let mut tick_join_handle = Box::pin(scope.spawn(state.tick()));
        loop {
            tokio::select! {
                _ = &mut tick_join_handle => {
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
