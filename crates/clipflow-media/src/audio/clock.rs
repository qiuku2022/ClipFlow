use crate::audio::transport::TransportState;
use clipflow_common::RationalTime;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockAnchor {
    pub dac_pos_ns: u64,
    pub qpc_ns: u64,
    pub period_ns: u64,
}

pub trait MasterClockProvider: Send + Sync {
    fn now_ns(&self) -> u64;
    fn now_rational(&self, timebase: u32) -> RationalTime {
        let ns = self.now_ns();
        let seconds = ns as f64 / 1_000_000_000.0;
        RationalTime::from_seconds(seconds, timebase)
    }
    fn set_state(&self, state: TransportState);
    fn state(&self) -> TransportState;
    fn seek(&self, target_ns: u64);
}

pub struct MonotonicClampedClock {
    base_instant: Instant,
    seq: AtomicU32,
    anchor: [std::sync::RwLock<ClockAnchor>; 2],
    active_idx: AtomicU32,
    last_output_ns: AtomicU64,
    paused_ns: AtomicU64,
    state: std::sync::RwLock<TransportState>,
}

impl MonotonicClampedClock {
    pub fn new(period_ns: u64) -> Self {
        let initial_anchor = ClockAnchor {
            dac_pos_ns: 0,
            qpc_ns: 0,
            period_ns,
        };
        Self {
            base_instant: Instant::now(),
            seq: AtomicU32::new(0),
            anchor: [
                std::sync::RwLock::new(initial_anchor),
                std::sync::RwLock::new(initial_anchor),
            ],
            active_idx: AtomicU32::new(0),
            last_output_ns: AtomicU64::new(0),
            paused_ns: AtomicU64::new(0),
            state: std::sync::RwLock::new(TransportState::Paused),
        }
    }

    pub fn current_qpc_ns(&self) -> u64 {
        self.base_instant.elapsed().as_nanos() as u64
    }

    pub fn update_anchor(&self, anchor: ClockAnchor) {
        let current_active = self.active_idx.load(Ordering::Acquire);
        let next_idx = (current_active ^ 1) as usize;

        // 写入非活动槽位
        *self.anchor[next_idx].write().unwrap() = anchor;

        // 翻转活动指针
        self.active_idx.store(next_idx as u32, Ordering::Release);
        self.seq.fetch_add(1, Ordering::Release);
    }
}

impl MasterClockProvider for MonotonicClampedClock {
    fn now_ns(&self) -> u64 {
        let state = *self.state.read().unwrap();
        match state {
            TransportState::Paused => self.paused_ns.load(Ordering::Acquire),
            TransportState::Scrubbing | TransportState::Seeking => {
                self.paused_ns.load(Ordering::Acquire)
            }
            TransportState::Playing => {
                let active = self.active_idx.load(Ordering::Acquire) as usize;
                let anchor = *self.anchor[active].read().unwrap();

                let qpc_now = self.current_qpc_ns();
                let elapsed = qpc_now.saturating_sub(anchor.qpc_ns);
                let max_clamp = (anchor.period_ns as f64 * 1.5) as u64;
                let clamped_elapsed = elapsed.min(max_clamp);
                let candidate = anchor.dac_pos_ns.saturating_add(clamped_elapsed);

                // CAS 保证绝对单调递增，杜绝时间倒流
                loop {
                    let last = self.last_output_ns.load(Ordering::Acquire);
                    let target = last.max(candidate);
                    if self
                        .last_output_ns
                        .compare_exchange_weak(last, target, Ordering::AcqRel, Ordering::Relaxed)
                        .is_ok()
                    {
                        return target;
                    }
                }
            }
        }
    }

    fn set_state(&self, new_state: TransportState) {
        let mut st = self.state.write().unwrap();
        if *st == TransportState::Playing && new_state != TransportState::Playing {
            // 暂停时刻冻结当前时间
            let current = self.now_ns();
            self.paused_ns.store(current, Ordering::Release);
        }
        *st = new_state;
    }

    fn state(&self) -> TransportState {
        *self.state.read().unwrap()
    }

    fn seek(&self, target_ns: u64) {
        self.paused_ns.store(target_ns, Ordering::Release);
        self.last_output_ns.store(target_ns, Ordering::Release);
        let qpc = self.current_qpc_ns();
        self.update_anchor(ClockAnchor {
            dac_pos_ns: target_ns,
            qpc_ns: qpc,
            period_ns: 10_000_000,
        });
    }
}
