use crate::timeline::galley_cache::SubtitleGalleyCache;
use crate::timeline::ruler::TimelineRuler;
use crate::timeline::viewport_culling::{TimelineViewport, TwoDimensionalCuller};
use clipflow_common::RationalTime;
use clipflow_timeline::models::Sequence;

pub struct SharedTimelineCanvas {
    ruler: TimelineRuler,
    galley_cache: SubtitleGalleyCache,
    pixels_per_second: f32,
    scroll_x_secs: f64,
    scroll_y_px: f32,
}

impl Default for SharedTimelineCanvas {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedTimelineCanvas {
    pub fn new() -> Self {
        Self {
            ruler: TimelineRuler::new(),
            galley_cache: SubtitleGalleyCache::new(1024),
            pixels_per_second: 50.0,
            scroll_x_secs: 0.0,
            scroll_y_px: 0.0,
        }
    }

    pub fn ruler(&self) -> &TimelineRuler {
        &self.ruler
    }

    pub fn galley_cache_mut(&mut self) -> &mut SubtitleGalleyCache {
        &mut self.galley_cache
    }

    pub fn pixels_per_second(&self) -> f32 {
        self.pixels_per_second
    }

    pub fn set_pixels_per_second(&mut self, pps: f32) {
        self.pixels_per_second = pps.clamp(1.0, 1000.0);
    }

    pub fn zoom_at(&mut self, factor: f32, anchor_sec: f64) {
        let old_pps = self.pixels_per_second;
        let new_pps = (old_pps * factor).clamp(1.0, 1000.0);
        // 保持 anchor_sec 在当前屏幕相对位置不变
        self.scroll_x_secs = anchor_sec - (anchor_sec - self.scroll_x_secs) * (old_pps / new_pps) as f64;
        self.pixels_per_second = new_pps;
    }

    pub fn scroll_x_secs(&self) -> f64 {
        self.scroll_x_secs
    }

    pub fn set_scroll_x_secs(&mut self, secs: f64) {
        self.scroll_x_secs = secs.max(0.0);
    }

    pub fn build_viewport(&self, viewport_width: f32, viewport_height: f32, timebase: u32) -> TimelineViewport {
        let visible_duration = viewport_width / self.pixels_per_second.max(1.0);
        let start = RationalTime::from_seconds(self.scroll_x_secs, timebase);
        let end = RationalTime::from_seconds(self.scroll_x_secs + visible_duration as f64, timebase);

        TimelineViewport {
            visible_time_start: start,
            visible_time_end: end,
            scroll_y: self.scroll_y_px,
            viewport_height,
            pixels_per_second: self.pixels_per_second,
        }
    }

    pub fn cull_visible_clips<'a>(&self, sequence: &'a Sequence, viewport: &TimelineViewport) -> Vec<(usize, &'a clipflow_timeline::models::Clip)> {
        let track_heights: Vec<f32> = sequence
            .tracks
            .iter()
            .map(|t| match t.kind {
                clipflow_timeline::models::TrackKind::Audio => 72.0,
                _ => 64.0,
            })
            .collect();
        let culler = TwoDimensionalCuller::new(&track_heights);
        culler.cull(sequence, viewport)
    }
}
