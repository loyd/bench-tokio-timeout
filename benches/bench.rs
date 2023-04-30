use std::time::Instant;

use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

use bench_tokio_reuse_sleep as lib;

macro_rules! bench {
    ($name:ident) => {
        fn $name(c: &mut Criterion) {
            c.bench_function(stringify!($name), |b| {
                b.iter_custom(|iters| {
                    let rt = Runtime::new().unwrap();
                    let expected_sum = (0..iters).sum::<u64>();

                    #[cfg(feature = "quanta")]
                    quanta::Instant::now();

                    let start = Instant::now();

                    // We spawn different tasks and use bounded queue with capacity 1
                    // in order to have more fair benchmark.
                    // If the channel is prefilled with all values,
                    // `timeout` has advantage because it never registers a timer.
                    let actual_sum = rt.block_on(async move {
                        let (tx, rx) = tokio::sync::mpsc::channel(1);
                        tokio::spawn(async move {
                            for i in 0..iters {
                                tx.send(i).await.unwrap();
                            }
                            drop(tx);
                        });
                        tokio::spawn(async move { lib::$name(rx).await })
                            .await
                            .unwrap()
                    });
                    assert_eq!(actual_sum, expected_sum);
                    start.elapsed()
                })
            });
        }
    };
}

bench!(baseline);
bench!(timeout);
bench!(sleep);
bench!(reused_sleep);
bench!(reused_boxed_sleep);

criterion_group!(
    benches,
    baseline,
    timeout,
    sleep,
    reused_sleep,
    reused_boxed_sleep
);
criterion_main!(benches);
