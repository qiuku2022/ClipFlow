use crate::audio::clock::ClockAnchor;

pub struct WasapiHardwareAnchor {
    sample_rate: u32,
}

impl WasapiHardwareAnchor {
    pub fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    pub fn capture_anchor(&self, device_position_samples: u64, qpc_ns: u64, period_ns: u64) -> ClockAnchor {
        let dac_pos_ns = (device_position_samples as f64 / self.sample_rate as f64 * 1_000_000_000.0) as u64;
        ClockAnchor {
            dac_pos_ns,
            qpc_ns,
            period_ns,
        }
    }
}
