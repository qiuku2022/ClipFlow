use std::time::{Duration, Instant};

pub struct AudioWatchdog {
    timeout_duration: Duration,
    last_heartbeat: Instant,
    fallback_active: bool,
}

impl AudioWatchdog {
    pub fn new(timeout_duration: Duration) -> Self {
        Self {
            timeout_duration,
            last_heartbeat: Instant::now(),
            fallback_active: false,
        }
    }

    pub fn tick_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
        self.fallback_active = false;
    }

    pub fn check_timeout(&mut self) -> bool {
        if self.last_heartbeat.elapsed() > self.timeout_duration {
            self.fallback_active = true;
            true
        } else {
            false
        }
    }

    pub fn is_fallback_active(&self) -> bool {
        self.fallback_active
    }
}
