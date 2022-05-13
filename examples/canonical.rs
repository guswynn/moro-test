use std::time::Duration;

use tokio::time::{interval, sleep};

struct StateKeeper {
    // Must be kept even
    state: u64,
}

impl StateKeeper {
    async fn tick(&mut self) {
        // intermediate state change
        self.state += 1;

        // pretend to do some io
        sleep(Duration::from_millis(1100)).await;

        // restore state to
        self.state += 1;
    }

    async fn evaluate(&mut self) {
        assert!(self.state % 2 == 0);
        println!("evaluating");
    }
}

async fn core_loop() {
    let mut state = StateKeeper { state: 4 };

    let mut progress_interval = interval(Duration::from_millis(5000));

    {
        // clear the first immediate interval
        progress_interval.tick().await;
    }

    loop {
        tokio::select! {
            _ = state.tick() => {
                state.evaluate().await;
            },
            _ = progress_interval.tick() => {
                println!("making progress!!");
            }

        }
    }
}

#[tokio::main]
async fn main() {
    core_loop().await;
}
