# bench-tokio-timeout

This repository contains a simple benchmark for `tokio::time::timeout()` and related constructions.

The benchmark checks the following use case: a task reads a channel and wraps every `recv()` into a timeout. Note: the timer is constantly rescheduled and never actually fires. Thus the test measures the overhead of timers. It's typical when we want to send heartbeats if the stream becomes slower or resubscribe to the data.

Implementations:
* `baseline` contain no timeouts at all.
* `timeout` uses `tokio::time::timeout()` directly. A timer entry is created on every message.
* `sleep` uses `tokio::time::Sleep`. A timer entry is created on every message.
* `reused_sleep` uses `Sleep` but resets the timer instead of building it on every call.

Note that the benchmark doesn't check the problem of [big size of futures](https://github.com/tokio-rs/tokio/issues/5348), because the timeout is never moved. So, in other cases, `reused_*` variants can be more performant.

The benchmark can be run with `--features quanta` to use `quanta::Instant` instead of `tokio::time::Instant`.

## Results
Benchmarks were performed on an [AMD Ryzen 7 4800HS CPU](https://en.wikichip.org/wiki/amd/ryzen_9/3900) with TSC as a clocksource.

```
group                   tokio-1.40                               tokio-1.40-quanta
-----                   ----------                               -----------------
timeout/baseline        1.01  486.4±117.75ns 2007.8 KElem/sec    1.00   481.5±92.26ns 2028.0 KElem/sec
timeout/reused_sleep    1.14  605.9±168.39ns 1611.7 KElem/sec    1.00  532.1±134.60ns 1835.3 KElem/sec
timeout/sleep           1.06  747.1±153.93ns 1307.2 KElem/sec    1.00  702.4±181.51ns 1390.4 KElem/sec
timeout/timeout         1.00  759.1±204.54ns 1286.5 KElem/sec
```

Note that `timeout()` cannot be used with `quanta`.
