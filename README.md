# moro-test

This repo attempts to document and explain some common issues with
building _concurrent_ and _parallel_ programs with `Future`s, and
uses the [moro](https://github.com/nikomatsakis/moro) crate to
attempt to use _structured concurrency_ to 

## Background 

`Future`s, in Rust, are a useful tool used to implement
relatively easy to read about concurrent programs. However,
there are some subtleties:

- `Future`s in Rust implement cancellation by being unconditionally
cancellable at _any_ yield point.
- Patterns build on top of `Future`s to add concurrency typically have
limitations:
 - `FuturesUnordered` and friends have complex performance characteristics,
 and have a limited API
 - _tasks_, as implemented by various runtimes/executors, can be
 structured and composed in complex ways, but come with a `'static`
 bound.
 - _tasks_ are the only standard way to build _parallelism_ on top of
 concurrency.

`moro` is an implementation of _structured concurrency_, which lets you compose
lightweight _tasks_ in local, _scoped_ 


## Cancellation-safety

`tokio::select!`, `futures::select!`, and others, are similar macros that are
commonly used to compose multiple futures together. This crate contains multiple
examples to show off a common issue with common usages of these primitives, and
builds on top of `moro` to try to imagine a world without that issue.

People often write `async` functions that are not _cancel-safe_, that is, they
implicitly expect that some intermediate state throughout the entirety of their
execution will not be observable by anyone else. This is a natural extension
of the fact that, baring panics, _synchronous_ code in Rust typically have
this property.

```
cargo run --example canonical # runs `examples/canonical.rs`
```
shows a minimized typical usage of `select!`,
where some intermediate state can be erroneously observed. While this example is
organized such that the examples always panics, in practice, these issues can
occur:
- instantaneously
- rare but regularly
- randomly
and can be exceedingly difficult to debug. In cases where the bugs are rare,
oftentimes refactoring the problem away is big undertaking.

```
cargo run --example canonical_moro # runs `examples/canonical_moro.rs`
```
imagines a world where `select!` macros ONLY allow allow "join handles" of spawned
`moro` "tasks", which unambiguously causes a compiler error. This compiler error is
a typical borrow-checker error: it *correctly* claims that you should not
expect to be able to mutate `StateKeeper` in `StateKeeper::evaluate`, while
the state could also be mutated by the `StateKeeper::tick` future.

```
cargo run --examples canonical_moro_fix_locking_naive # runs `examples/canonical_moro_fix_locking_naive.rs`
```
shows what could happen if a user, presented with the above borrow-checker error,
could write to resolve that error. This uses a typical interior-mutability pattern.

Unfortunately this reveals 2 related problems:
- `moro` join handles, as implemented, are not strictly _cancel-safe_. See the comment
on the `progress_interval.tick()` call to see more information.
- The `progress_interval.tick()` racing with the spawned `StateKeeper::tick` can cause
`tick` to be called more times than `evaluate`. This is show with some log lines.

Its unclear to me at this point how these issues can be resolved.

```
cargo run --examples canonical_moro_fix_locking # runs `examples/canonical_moro_fix_locking.rs`
```
adapts `canonical_moro_fix_locking_naive` to use a local variable to store a
`moro` "join handle", to prevent erroneous drops.

Note that this uses `Arc` and `tokio` `Mutex`s, but there is nothing in the `moro` implementation
that leads me to think that thread-local versions of `spawn` would work, allowing the use
of `Rc` and `RefCell` for interior mutability
(See [this branch](https://github.com/guswynn/moro/tree/local)).

```
cargo run --examples canonical_moro_fix_ownership # runs `examples/canonical_moro_fix_ownership.rs`
```
uses ownership semantics to avoid both erroneous drops AND the use of interior mutability.
This is very likely not always possible for many real-world examples.

### Conclusion
It would be nice to be able to push users as aggresively as possible to the last 2 examples here.
Structured, local concurrency 
