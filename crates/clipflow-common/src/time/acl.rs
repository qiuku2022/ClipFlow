use serde::{Deserialize, Serialize};
use super::{RationalTime, TimeRange};

/// Agent 外部通信使用的原始请求结构 (仅用于 IPC / JSON 序列化)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCutRequest {
    pub start_time_seconds: f64,
    pub end_time_seconds: f64,
    pub reason: String,
}

/// Agent 时间轴防腐网关
pub struct AgentTimelineAcl;

impl AgentTimelineAcl {
    /// 将外部浮点秒数严格量化吸附至序列当前帧网格分界点
    #[inline]
    pub fn seconds_to_snapped_time(
        seconds: f64,
        timebase: u32,
        fps_denominator: u32,
    ) -> RationalTime {
        if seconds <= 0.0 {
            return RationalTime::new(0, timebase);
        }
        let frame_duration_s = fps_denominator as f64 / timebase as f64;
        let frame_index = (seconds / frame_duration_s).round() as i64;
        let snapped_value = frame_index * (fps_denominator as i64);
        RationalTime::new(snapped_value, timebase)
    }

    /// 对 Agent 提交的切除请求执行合法性校验、帧吸附与拓扑缝合
    /// 相邻切片间距 <= 1 帧微差时自动无缝对齐，保证切片空洞坏帧率严格为 0
    pub fn sanitize_and_stitch_cuts(
        raw_cuts: &[AgentCutRequest],
        source_duration: RationalTime,
        timebase: u32,
        fps_denominator: u32,
    ) -> Vec<TimeRange> {
        let mut valid_ranges: Vec<TimeRange> = Vec::with_capacity(raw_cuts.len());
        let max_ticks = source_duration.rescaled_to(timebase).value;

        // Ensure cuts are sorted by start time
        let mut sorted_cuts = raw_cuts.to_vec();
        sorted_cuts.sort_by(|a, b| a.start_time_seconds.partial_cmp(&b.start_time_seconds).unwrap());

        for cut in &sorted_cuts {
            let mut start = Self::seconds_to_snapped_time(cut.start_time_seconds, timebase, fps_denominator);
            let mut end = Self::seconds_to_snapped_time(cut.end_time_seconds, timebase, fps_denominator);

            // Clamp to source duration boundaries
            if start.value < 0 { start.value = 0; }
            if end.value > max_ticks { end.value = max_ticks; }
            if start.value >= end.value { continue; } // Invalid or zero-length cut

            // Topology stitching: check gap with previous valid range
            if let Some(last_range) = valid_ranges.last_mut() {
                let gap_ticks = start.value - (last_range.start.value + last_range.duration.value);
                // If gap is exactly 1 frame or less, stitch it
                if gap_ticks > 0 && gap_ticks <= (fps_denominator as i64) {
                    // Extend previous range to cover the gap
                    // Actually, if it's a cut to be removed, we might merge the cuts? 
                    // The requirement says: "相邻切片间距 <= 1 帧微差时自动无缝对齐，保证切片空洞坏帧率严格为 0"
                    // Wait, `AgentCutRequest` means we are cutting these OUT (removing them) or keeping them?
                    // "对 Agent 提交的切除请求执行合法性校验、帧吸附与拓扑缝合" (Cut requests)
                    // If we merge two adjacent cuts, we just extend the previous cut to the end of the new cut.
                    last_range.duration.value = end.value - last_range.start.value;
                    continue;
                }
            }

            valid_ranges.push(TimeRange::new(start, RationalTime::new(end.value - start.value, timebase)));
        }

        valid_ranges
    }
}
