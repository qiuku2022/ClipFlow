use crate::audio::peak_format::PeakRecord;

pub struct WaveformLodPyramid {
    lod0: Vec<PeakRecord>,
    lod1: Vec<PeakRecord>, // 5x
    lod2: Vec<PeakRecord>, // 25x
    sample_rate: u32,
    window_size: u32,
}

impl WaveformLodPyramid {
    pub fn new(records: Vec<PeakRecord>, sample_rate: u32, window_size: u32) -> Self {
        // 构建 LOD 1 (5 合 1)
        let lod1 = Self::downsample(&records, 5);
        // 构建 LOD 2 (25 合 1)
        let lod2 = Self::downsample(&records, 25);

        Self {
            lod0: records,
            lod1,
            lod2,
            sample_rate,
            window_size,
        }
    }

    fn downsample(input: &[PeakRecord], factor: usize) -> Vec<PeakRecord> {
        let count = input.len() / factor;
        let mut output = Vec::with_capacity(count);
        for chunk in input.chunks(factor) {
            let mut min_val = i16::MAX;
            let mut max_val = i16::MIN;
            let mut sum_rms: u32 = 0;
            for r in chunk {
                min_val = min_val.min(r.min_sample);
                max_val = max_val.max(r.max_sample);
                sum_rms += r.rms_energy as u32;
            }
            let avg_rms = (sum_rms / chunk.len() as u32) as u16;
            output.push(PeakRecord {
                min_sample: min_val,
                max_sample: max_val,
                rms_energy: avg_rms,
            });
        }
        output
    }

    /// 查询给定时间范围内的归一化波形幅度点集
    /// pixels_per_second: 每秒屏幕像素数，决定挑选哪一层 LOD
    pub fn query_range(&self, start_sec: f64, end_sec: f64, pixels_per_second: f32) -> Vec<(f32, f32)> {
        let records_per_sec = self.sample_rate as f64 / self.window_size as f64;

        // 根据每像素点跨度选择 LOD
        let (records, factor) = if pixels_per_second > 50.0 {
            (&self.lod0, 1.0)
        } else if pixels_per_second > 10.0 {
            (&self.lod1, 5.0)
        } else {
            (&self.lod2, 25.0)
        };

        let start_idx = ((start_sec * records_per_sec) / factor).floor() as usize;
        let end_idx = ((end_sec * records_per_sec) / factor).ceil() as usize;

        let clamped_start = start_idx.min(records.len());
        let clamped_end = end_idx.min(records.len());

        let mut points = Vec::with_capacity(clamped_end.saturating_sub(clamped_start));
        for r in &records[clamped_start..clamped_end] {
            let min_norm = r.min_sample as f32 / 32768.0;
            let max_norm = r.max_sample as f32 / 32768.0;
            points.push((min_norm, max_norm));
        }

        points
    }
}
