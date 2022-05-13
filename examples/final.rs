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

    async fn evaluate(&self) -> Result<(), ()> {
        self.evaluates.fetch_add(1, Ordering::SeqCst);
        let state = self.state.lock().await;

        assert!(*state % 2 == 0);
        println!(
            "evaluating, seen {} ticks and {} evaluates",
            self.ticks.load(Ordering::SeqCst),
            self.evaluates.load(Ordering::SeqCst)
        );

        if self.evaluates.load(Ordering::SeqCst) > 7 {
            println!("finished eval with error");
            return Err(());
        }
        Ok(())
    }

    async fn do_something_else(&self) {
        let mut state = self.state.lock().await;
        *state += 2;
    }
}

async fn core_loop() {
    let state = StateKeeper {
        state: Arc::new(Mutex::new(4)),
        ticks: AtomicUsize::default(),
        evaluates: AtomicUsize::default(),
    };

    let mut progress_interval = interval(Duration::from_millis(5000));

    {
        // clear the first immediate interval
        progress_interval.tick().await;
    }

    let scope = moro::async_scope!(|scope| {
        let tick = scope.spawn_cancelling(async {
            loop {
                state.tick().await;
                // inference fails here with `?` :(
                match state.evaluate().await {
                    Ok(()) => {}
                    Err(()) => {
                        return Err::<(), ()>(());
                    }
                }
            }
        });
        let timer = scope.spawn(async {
            loop {
                progress_interval.tick().await;
                println!("doing something else!");
                state.do_something_else().await;
            }
        });

        futures::future::join(tick, timer).await;
    });

    match scope.await {
        Ok(()) => {}
        Err(()) => println!("done"),
    }
}

#[tokio::main]
async fn main() {
    core_loop().await;
}
