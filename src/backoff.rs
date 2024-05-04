use std::time::{Duration, Instant};

use tokio::time::sleep;

pub struct BackOff {
    /// The last time we had an error.
    attempt_start: Instant,
    /// The number of errors we've had in a row.
    error_count: u32,
}

impl Default for BackOff {
    fn default() -> Self {
        Self {
            attempt_start: Instant::now(),
            error_count: 0,
        }
    }
}

impl BackOff {
    /// Don't retry too fast on errors.
    pub async fn back_off(&mut self) {
        /// Don't let the delay grow too much.
        const MAX_ERROR_COUNT: u32 = 10;
        const BASE_DELAY: Duration = Duration::from_secs(1);
        const BACKOFF_FACTOR: f64 = 1.5;
        /// If we run this long without an error, reset the delay.
        const RECOVERY_TIME: Duration = Duration::from_secs(60);

        if self.attempt_start.elapsed() > RECOVERY_TIME {
            self.error_count = 0;
        }

        let delay =
            BASE_DELAY * BACKOFF_FACTOR.powi(self.error_count.min(MAX_ERROR_COUNT) as i32) as u32;

        sleep(delay).await;

        // Set delay last, so we don't count the time it takes to sleep.
        self.attempt_start = Instant::now();
    }
}
