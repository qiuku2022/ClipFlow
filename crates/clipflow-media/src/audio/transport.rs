#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportState {
    Playing,
    Paused,
    Scrubbing,
    Seeking,
}

pub struct TransportController {
    state: TransportState,
    scrub_target_ns: u64,
}

impl Default for TransportController {
    fn default() -> Self {
        Self::new()
    }
}

impl TransportController {
    pub fn new() -> Self {
        Self {
            state: TransportState::Paused,
            scrub_target_ns: 0,
        }
    }

    pub fn state(&self) -> TransportState {
        self.state
    }

    pub fn play(&mut self) {
        self.state = TransportState::Playing;
    }

    pub fn pause(&mut self) {
        self.state = TransportState::Paused;
    }

    pub fn toggle_play_pause(&mut self) {
        if self.state == TransportState::Playing {
            self.pause();
        } else {
            self.play();
        }
    }

    pub fn start_scrubbing(&mut self, target_ns: u64) {
        self.state = TransportState::Scrubbing;
        self.scrub_target_ns = target_ns;
    }

    pub fn update_scrub(&mut self, target_ns: u64) {
        self.scrub_target_ns = target_ns;
    }

    pub fn stop_scrubbing(&mut self) {
        self.state = TransportState::Paused;
    }

    pub fn scrub_target_ns(&self) -> u64 {
        self.scrub_target_ns
    }
}
