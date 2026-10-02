use crate::audio::peak_format::{PeakFileHeader, PeakRecord};

pub struct PeakGenerator {
    sample_rate: u32,
    channels: u16,
    window_size: u32,
}

impl PeakGenerator {
    pub fn new(sample_rate: u32, channels: u16, window_size: u32) -> Self {
        Self {
            sample_rate,
            channels,
            window_size,
        }
    }

    pub fn generate_from_i16_interleaved(&self, samples: &[i16]) -> Vec<u8> {
        let ch = self.channels as usize;
        let win = self.window_size as usize;
        let total_frames = samples.len() / ch;
        let num_windows = total_frames / win;

        let mut output = Vec::with_capacity(16 + num_windows * 6);
        let header = PeakFileHeader::new(self.sample_rate, self.channels, self.window_size);
        output.extend_from_slice(&header.to_bytes());

        for w in 0..num_windows {
            let start_frame = w * win;
            let end_frame = start_frame + win;

            let mut min_val = i16::MAX;
            let mut max_val = i16::MIN;
            let mut sum_sq: f64 = 0.0;

            for f in start_frame..end_frame {
                // 取多通道绝对峰值
                for c in 0..ch {
                    let s = samples[f * ch + c];
                    if s < min_val {
                        min_val = s;
                    }
                    if s > max_val {
                        max_val = s;
                    }
                    sum_sq += (s as f64) * (s as f64);
                }
            }

            let rms = ((sum_sq / (win * ch) as f64).sqrt() as u16).min(u16::MAX);
            let record = PeakRecord {
                min_sample: min_val,
                max_sample: max_val,
                rms_energy: rms,
            };
            output.extend_from_slice(&record.to_bytes());
        }

        output
    }
}
