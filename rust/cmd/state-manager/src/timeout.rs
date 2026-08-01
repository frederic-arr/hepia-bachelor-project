use std::time::{Duration, Instant};

use thiserror::Error;

#[derive(Error, Debug)]
#[error("operation timed out")]
pub struct TimedOut {
    _opaque: (),
}

pub trait Timeout<T>: Sized {
    fn timeout_in(
        self,
        duration: Duration,
    ) -> impl Future<Output = Result<T, TimedOut>>;
    fn timeout_at(
        self,
        deadline: Instant,
    ) -> impl Future<Output = Result<T, TimedOut>>;

    fn timeout_milis(
        self,
        milis: u64,
    ) -> impl Future<Output = Result<T, TimedOut>> {
        self.timeout_in(Duration::from_millis(milis))
    }

    fn timeout_secs(
        self,
        secs: u64,
    ) -> impl Future<Output = Result<T, TimedOut>> {
        self.timeout_in(Duration::from_secs(secs))
    }

    fn timeout_secs_f32(
        self,
        secs: f32,
    ) -> impl Future<Output = Result<T, TimedOut>> {
        self.timeout_in(Duration::from_secs_f32(secs))
    }

    fn timeout_secs_f64(
        self,
        secs: f64,
    ) -> impl Future<Output = Result<T, TimedOut>> {
        self.timeout_in(Duration::from_secs_f64(secs))
    }
}

impl<T, U> Timeout<T> for U
where
    U: Future<Output = T>,
{
    fn timeout_in(
        self,
        duration: Duration,
    ) -> impl Future<Output = Result<T, TimedOut>> {
        timeout_in(duration, self)
    }

    fn timeout_at(
        self,
        deadline: Instant,
    ) -> impl Future<Output = Result<T, TimedOut>> {
        timeout_at(deadline, self)
    }
}

pub fn timeout_in<T, F>(
    duration: Duration,
    f: F,
) -> impl Future<Output = Result<T, TimedOut>>
where
    F: Future<Output = T>,
{
    let deadline = Instant::now() + duration;
    timeout_at(deadline, f)
}

pub async fn timeout_at<T, F>(deadline: Instant, f: F) -> Result<T, TimedOut>
where
    F: Future<Output = T>,
{
    tokio::time::timeout_at(deadline.into(), f)
        .await
        .or(Err(TimedOut { _opaque: () }))
}
