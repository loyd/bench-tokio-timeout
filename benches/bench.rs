use std::time::Instant;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use tokio::runtime::Runtime;

macro_rules! bench_impl {
    ($group:ident, $name:ident) => {
        $group.bench_function(stringify!($name), |b| {
            b.iter_custom(|iters| {
                let rt = Runtime::new().unwrap();
                let expected_sum = (0..iters).sum::<u64>();

                let start = Instant::now();

                // We spawn different tasks and use bounded queue with capacity 1
                // in order to have more fair benchmark. Otherwise, `Timeout` and
                // If the channel is prefilled with all values, `Timeout` and `Sleep`
                // don't have a change to be registered.
                let actual_sum = rt.block_on(async move {
                    let (tx, rx) = tokio::sync::mpsc::channel(1);
                    tokio::spawn(async move {
                        for i in 0..iters {
                            tx.send(i).await.unwrap();
                        }
                        drop(tx);
                    });
                    tokio::spawn(async move { bench_tokio_sleep::$name(rx).await })
                        .await
                        .unwrap()
                });
                assert_eq!(actual_sum, expected_sum);
                start.elapsed()
            })
        });
    };
}

fn bench(c: &mut Criterion) {
    // Perform calibration before any real benches.
    #[cfg(feature = "quanta")]
    quanta::Instant::now();

    let mut group = c.benchmark_group("timeout");
    group.throughput(Throughput::Elements(1));

    bench_impl!(group, baseline);
    #[cfg(not(feature = "quanta"))]
    bench_impl!(group, timeout);
    bench_impl!(group, sleep);
    bench_impl!(group, reused_sleep);
}

criterion_group!(benches, bench);
criterion_main!(benches);
