use std::time::Instant;

pub use std::time::Duration;

/// Converts a hertz to a duration
pub fn hz_to_duration(hz: u32) -> Duration {
    Duration::from_secs(1) / hz
}

/// Stopwatch
pub struct Stopwatch {
    start: Instant,
}

impl Stopwatch {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Peeks the duration since the last stop
    pub fn peek(&self) -> Duration {
        Instant::now() - self.start
    }

    /// Returns the elapsed time since the stopwatch was last checked
    pub fn elapsed(&mut self) -> Duration {
        let now = Instant::now();
        let diff = now - self.start;
        self.start = now;

        diff
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
