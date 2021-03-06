use std::time::{Duration, Instant};

use futures::future::join;

async fn asterisk_work_asterisk(s: &str) {
    std::thread::sleep(Duration::from_millis(100));
    println!("access stack value: {}", s);
}

async fn bench(parallel: bool, ub: bool) {
    let now = Instant::now();
    let s = "stackkk".to_string();
    let scope = moro::async_scope!(|scope| {
        for _ in 1..10 {
            if parallel {
                unsafe {
                    let job1 = scope.spawn_parallel(asterisk_work_asterisk(&s));
                    let job2 = scope.spawn_parallel(asterisk_work_asterisk(&s));
                    join(job1, job2).await;
                }
            } else {
                let job1 = scope.spawn(asterisk_work_asterisk(&s));
                let job2 = scope.spawn(asterisk_work_asterisk(&s));

                join(job1, job2).await;
            }
        }
    })
    .infallible();

    tokio::pin!(scope);

    if ub {
        // Poll a few times to spawn some tasks, but fail
        // to await till completion. We don't even
        // need to `forget` the `scope`, as nothing in its
        // drop impl will cancel and await the tokio tasks.
        use futures::future::FutureExt;
        (&mut scope).now_or_never();
        (&mut scope).now_or_never();
        (&mut scope).now_or_never();

        // A note on why this is ub, and how it may be fixable:
        //
        // The tokio tasks spawned above require that this future is
        // not destroyed until they finish. The `.await` below does
        // just that, but failing to _fully_ `await`
        // the `ScopeBody` future, (or, something like a panic)
        // requires that we wait for the sub-tasks to be done.
        // I believe this may require `AsyncDrop`.
        //
        // The problem with that is that `std::mem::forget`,
        // a reference-cycle, etc. may cause that drop impl
        // to not called.
        // TODO(guswynn): update this documentation
        // as I obtain more information.
    } else {
        scope.await
    }

    println!("Elapsed: {:?}", now.elapsed());
}

// helper method to try to clobber the `bench` stack
fn fill_stack(_thing: usize, _thing2: usize) {
    std::thread::sleep(Duration::from_millis(500))
}

#[derive(clap::Parser)]
struct Args {
    /// Run with tokio tasks
    #[clap(long)]
    parallel: bool,
    /// Exercise ub (no ub if you dont use `--parallel`)
    #[clap(long)]
    ub: bool,
}

#[tokio::main]
async fn main() {
    use clap::Parser;
    let args = Args::parse();
    bench(args.parallel, args.ub).await;
    if args.ub {
        fill_stack(7, 8);
    }
}
