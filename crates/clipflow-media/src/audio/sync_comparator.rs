#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAction {
    RenderCurrentFrame,
    HoldPreviousFrame,
    DropCurrentFrame,
}

pub struct HysteresisSyncComparator {
    in_lock: bool,
    exit_threshold_ns: i64,    // 12ms = 12_000_000 ns
    recover_threshold_ns: i64, // 8ms = 8_000_000 ns
    drop_threshold_ns: i64,    // -40ms = -40_000_000 ns
    vsync_forward_ns: i64,     // 8.33ms @ 60Hz
}

impl Default for HysteresisSyncComparator {
    fn default() -> Self {
        Self::new()
    }
}

impl HysteresisSyncComparator {
    pub fn new() -> Self {
        Self {
            in_lock: true,
            exit_threshold_ns: 12_000_000,
            recover_threshold_ns: 8_000_000,
            drop_threshold_ns: -40_000_000,
            vsync_forward_ns: 8_333_333,
        }
    }

    pub fn is_in_lock(&self) -> bool {
        self.in_lock
    }

    /// 输入 delta_ns = PTS - (AudioClockNow + VsyncForward)
    pub fn evaluate(&mut self, delta_ns: i64) -> SyncAction {
        if self.in_lock {
            if delta_ns.abs() > self.exit_threshold_ns {
                self.in_lock = false;
            }
        } else if delta_ns.abs() <= self.recover_threshold_ns {
            self.in_lock = true;
        }

        if self.in_lock {
            SyncAction::RenderCurrentFrame
        } else if delta_ns > self.recover_threshold_ns {
            SyncAction::HoldPreviousFrame
        } else if delta_ns < self.drop_threshold_ns {
            SyncAction::DropCurrentFrame
        } else {
            // 滞后追赶
            SyncAction::RenderCurrentFrame
        }
    }

    pub fn vsync_forward_ns(&self) -> i64 {
        self.vsync_forward_ns
    }
}
