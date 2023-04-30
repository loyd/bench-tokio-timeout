# bench-tokio-timeout

This repository contains a simple benchmark for `tokio::time::timeout()` and related constructions.

The benchmark checks the following use case: a task reads a channel and wraps every `recv()` into a timeout. Note: the timer is constantly rescheduled and never actually fires. Thus the test measures the overhead of timers. It's typical when we want to send heartbeats if the stream becomes slower or resubscribe to the data.

Implementations:
* `baseline` contain no timeouts at all.
* `timeout` uses `tokio::time::timeout()` directly. A timer entry is created on every message.
* `sleep` uses `tokio::time::Sleep`. A timer entry is created on every message.
* `reused_sleep` uses `Sleep` but resets the timer instead of building it on every call.
* `reused_boxed_sleep` uses `Pin<Box<Sleep>>`.

Note that the benchmark doesn't check the problem of [big size of futures](https://github.com/tokio-rs/tokio/issues/5348), because the timeout is never moved. So, in other cases, `reused_*` variants can be more performant.

The most expensive part is `Instant::now`. The benchmark can be run with `--features quanta` to use `quanta::Instant` instead. See results to see this significant difference.

## Results
Benchmarks were performed on an [AMD Ryzen 7 4800HS CPU](https://en.wikichip.org/wiki/amd/ryzen_9/3900).

```
group                         tokio-1.28                               tokio-1.28-quanta
-----                         ----------                               -----------------
timeout/baseline              1.00  356.6±116.62ns  2.7 MElem/sec      1.02  364.8±119.89ns  2.6 MElem/sec
timeout/reused_boxed_sleep    3.65  1714.5±302.61ns 569.6 KElem/sec    1.00  470.3±145.91ns  2.0 MElem/sec
timeout/reused_sleep          3.45  1643.4±265.58ns 594.2 KElem/sec    1.00  476.4±154.02ns  2.0 MElem/sec
timeout/sleep                 3.61  1789.7±324.50ns 545.7 KElem/sec    1.00  496.1±126.25ns 1968.3 KElem/sec
timeout/timeout               1.00  1768.7±325.27ns 552.1 KElem/sec
```

Note that `timeout()` cannot be used with `quanta`.

## Summary
* All timeouts are incredibly slow (up to x7) comparing to the baseline if used with tokio's `Instant`.
* `quanta::Instant` increases performance dramatically.
* Building a timer entry is actually cheap enough (~7% overhead).
