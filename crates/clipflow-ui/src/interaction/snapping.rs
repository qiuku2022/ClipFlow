use clipflow_common::RationalTime;

/// 磁吸对齐引擎
#[derive(Debug, Clone)]
pub struct SnapEngine {
    enabled: bool,
}

impl Default for SnapEngine {
    fn default() -> Self {
        Self::new(true)
    }
}

impl SnapEngine {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Shift 临时反转状态：
    /// 当全局开启且按住 Shift 时关闭；当全局关闭且按住 Shift 时开启
    pub fn is_active_with_shift(&self, shift_pressed: bool) -> bool {
        if shift_pressed {
            !self.enabled
        } else {
            self.enabled
        }
    }

    /// 计算给定位置是否命中吸附目标
    /// 如果在 radius 范围内找到最近的吸附目标，则返回吸附后的目标时间；否则返回原始 current
    pub fn snap_time(
        &self,
        current: RationalTime,
        targets: &[RationalTime],
        radius: RationalTime,
    ) -> RationalTime {
        let mut closest_target: Option<RationalTime> = None;
        let mut min_diff: Option<RationalTime> = None;

        for target in targets {
            let diff = if current >= *target {
                current - *target
            } else {
                *target - current
            };

            if diff <= radius {
                match min_diff {
                    Some(cur_min) => {
                        if diff < cur_min {
                            min_diff = Some(diff);
                            closest_target = Some(*target);
                        }
                    }
                    None => {
                        min_diff = Some(diff);
                        closest_target = Some(*target);
                    }
                }
            }
        }

        closest_target.unwrap_or(current)
    }
}
