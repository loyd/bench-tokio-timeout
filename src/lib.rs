use tokio::select;
use tokio::sync::mpsc::Receiver;
use tokio::time::{self, Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(10);

pub async fn baseline(mut rx: Receiver<u64>) -> u64 {
    let mut sum = 0;

    while let Some(no) = rx.recv().await {
        sum += no;
    }

    sum
}

#[cfg(not(feature = "quanta"))] // `timeout()` cannot be used with `quanta`
pub async fn timeout(mut rx: Receiver<u64>) -> u64 {
    let mut sum = 0;

    loop {
        match time::timeout(TIMEOUT, rx.recv()).await {
            Ok(Some(no)) => sum += no,
            Ok(None) => break,
            Err(_) => panic!("timeout"),
        }
    }

    sum
}

pub async fn sleep(mut rx: Receiver<u64>) -> u64 {
    let mut sum = 0;

    loop {
        select! {
            res = rx.recv() => match res {
                Some(no) => sum += no,
                None => break,
            },
            () = time::sleep_until(after(TIMEOUT)) => panic!("timeout"),
        }
    }

    sum
}

pub async fn reused_sleep(mut rx: Receiver<u64>) -> u64 {
    let mut sum = 0;
    let sleep = time::sleep_until(after(TIMEOUT));
    tokio::pin!(sleep);

    loop {
        select! {
            res = rx.recv() => match res {
                Some(no) => {
                    sum += no;
                    sleep.as_mut().reset(after(TIMEOUT));
                },
                None => break,
            },
            () = &mut sleep => panic!("timeout"),
        }
    }

    sum
}

pub async fn reused_boxed_sleep(mut rx: Receiver<u64>) -> u64 {
    let mut sum = 0;
    let mut sleep = Box::pin(time::sleep_until(after(TIMEOUT)));

    loop {
        select! {
            res = rx.recv() => match res {
                Some(no) => {
                    sum += no;
                    sleep.as_mut().reset(after(TIMEOUT));
                },
                None => break,
            },
            () = &mut sleep => panic!("timeout"),
        }
    }

    sum
}

#[cfg(feature = "quanta")]
fn after(duration: Duration) -> Instant {
    thread_local! {
        static ORIGIN: (Instant, quanta::Instant) = (Instant::now(), quanta::Instant::now());
    }

    ORIGIN.with(|origin| origin.0 + (quanta::Instant::now() + duration - origin.1))
}

#[cfg(not(feature = "quanta"))]
fn after(duration: Duration) -> Instant {
    Instant::now() + duration
}
