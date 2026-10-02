use clipflow_common::RationalTime;
use clipflow_timeline::models::{Clip, Sequence};

pub struct TimelineViewport {
    pub visible_time_start: RationalTime,
    pub visible_time_end: RationalTime,
    pub scroll_y: f32,
    pub viewport_height: f32,
    pub pixels_per_second: f32,
}

pub struct TwoDimensionalCuller {
    track_y_starts: Vec<f32>,
    track_heights: Vec<f32>,
    total_height: f32,
}

impl TwoDimensionalCuller {
    pub fn new(track_heights: &[f32]) -> Self {
        let mut track_y_starts = Vec::with_capacity(track_heights.len());
        let mut current_y = 0.0;
        for &h in track_heights {
            track_y_starts.push(current_y);
            current_y += h;
        }

        Self {
            track_y_starts,
            track_heights: track_heights.to_vec(),
            total_height: current_y,
        }
    }

    pub fn total_height(&self) -> f32 {
        self.total_height
    }

    pub fn track_heights(&self) -> &[f32] {
        &self.track_heights
    }

    /// 正交二维 AABB 剪裁：返回 (track_index, &Clip) 列表
    pub fn cull<'a>(&self, sequence: &'a Sequence, viewport: &TimelineViewport) -> Vec<(usize, &'a Clip)> {
        let mut visible_clips = Vec::new();
        if sequence.tracks.is_empty() || self.track_y_starts.is_empty() {
            return visible_clips;
        }

        // 1. Y 轴垂直二分粗筛 (带有 64px 缓冲裕量)
        let y_min = (viewport.scroll_y - 64.0).max(0.0);
        let y_max = viewport.scroll_y + viewport.viewport_height + 64.0;

        let start_track_idx = match self
            .track_y_starts
            .binary_search_by(|&y| y.partial_cmp(&y_min).unwrap())
        {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };

        let end_track_idx = match self
            .track_y_starts
            .binary_search_by(|&y| y.partial_cmp(&y_max).unwrap())
        {
            Ok(i) => (i + 1).min(sequence.tracks.len()),
            Err(i) => (i + 1).min(sequence.tracks.len()),
        };

        // 2. X 轴时间单调区间二分切片筛选 (带有 120px 缓冲裕量对应时间)
        let margin_secs = (120.0 / viewport.pixels_per_second.max(1.0)) as f64;
        let timebase = viewport.visible_time_start.timescale;
        let margin_ticks = (margin_secs * timebase as f64).round() as i64;

        let vis_start_with_margin = RationalTime::new(
            (viewport.visible_time_start.value - margin_ticks).max(0),
            timebase,
        );
        let vis_end_with_margin = RationalTime::new(
            viewport.visible_time_end.value + margin_ticks,
            timebase,
        );

        for t_idx in start_track_idx..end_track_idx {
            if t_idx >= sequence.tracks.len() {
                break;
            }
            let track = &sequence.tracks[t_idx];
            let clips = track.clips();

            // 二分定位首个与视口相交的切片
            let start_idx = clips.partition_point(|clip| {
                clip.timeline_range.end_exclusive().rescaled_to(timebase) <= vis_start_with_margin
            });

            for clip in &clips[start_idx..] {
                if clip.timeline_range.start.rescaled_to(timebase) >= vis_end_with_margin {
                    // 超出视口右边界，立即终止横向遍历
                    break;
                }
                visible_clips.push((t_idx, clip));
            }
        }

        visible_clips
    }
}
