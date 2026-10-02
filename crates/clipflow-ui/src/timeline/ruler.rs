use clipflow_common::RationalTime;

pub struct TimelineRuler {
    height: f32,
}

impl Default for TimelineRuler {
    fn default() -> Self {
        Self::new()
    }
}

impl TimelineRuler {
    pub fn new() -> Self {
        Self { height: 28.0 }
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    /// 根据每秒像素数与视口起始/结束时间计算主副刻度间隔
    pub fn compute_tick_interval(pixels_per_second: f32) -> f64 {
        if pixels_per_second > 200.0 {
            0.1 // 100ms
        } else if pixels_per_second > 50.0 {
            1.0 // 1s
        } else if pixels_per_second > 10.0 {
            5.0 // 5s
        } else {
            30.0 // 30s
        }
    }

    pub fn time_to_x(time: RationalTime, start_time: RationalTime, pixels_per_second: f32) -> f32 {
        let delta_secs = time.to_seconds() - start_time.to_seconds();
        (delta_secs as f32 * pixels_per_second).max(0.0)
    }

    pub fn x_to_time(x: f32, start_time: RationalTime, pixels_per_second: f32, timebase: u32) -> RationalTime {
        let delta_secs = x / pixels_per_second.max(1.0);
        let current_secs = start_time.to_seconds() + delta_secs as f64;
        RationalTime::from_seconds(current_secs, timebase)
    }
}
