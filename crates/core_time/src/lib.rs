pub type Duration = std::time::Duration;

/// Creates a duration from hz
pub fn duration_from_hz(hz: u32) -> Duration {
    let hz = {
        if hz == 0 {
            1
        } else {
            hz
        }
    };

    Duration::from_secs(1) / hz
}

/// A timer class
pub struct Timer {
    start: std::time::Instant,
}

impl Timer {
    /// Creates a new timer
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    /// Peeks the elapsed time for the timer
    pub fn elapsed(&self) -> Duration {
        std::time::Instant::now() - self.start
    }

    /// Stops the given timer, returning the elapsed time.
    pub fn stop(&mut self) -> Duration {
        let duration = self.elapsed();
        self.reset();
        duration
    }

    /// Resets the timer to the currant instant.
    pub fn reset(&mut self) {
        self.start = std::time::Instant::now();
    }
}
