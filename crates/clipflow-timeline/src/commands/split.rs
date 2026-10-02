use crate::commands::{CommandError, TimelineCommand};
use crate::models::sequence::Sequence;
use clipflow_common::{RationalTime, TimeRange};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SplitClipCommand {
    pub target_clip_id: Uuid,
    pub cut_point: RationalTime,
    created_clip_id: Option<Uuid>,
    original_timeline_duration: Option<RationalTime>,
    original_source_duration: Option<RationalTime>,
}

impl SplitClipCommand {
    pub fn new(target_clip_id: Uuid, cut_point: RationalTime) -> Self {
        Self {
            target_clip_id,
            cut_point,
            created_clip_id: None,
            original_timeline_duration: None,
            original_source_duration: None,
        }
    }

    pub fn created_clip_id(&self) -> Option<Uuid> {
        self.created_clip_id
    }
}

impl TimelineCommand for SplitClipCommand {
    fn execute(&mut self, sequence: &mut Sequence) -> Result<(), CommandError> {
        let (track, clip_idx) = sequence
            .find_clip_mut(self.target_clip_id)
            .ok_or(CommandError::ClipNotFound(self.target_clip_id))?;

        let orig_clip = &mut track.clips[clip_idx];
        let tl_start = orig_clip.timeline_range.start;
        let tl_end = orig_clip.timeline_range.end_exclusive();

        let cut_rescaled = self.cut_point.rescaled_to(tl_start.timescale);
        if cut_rescaled <= tl_start || cut_rescaled >= tl_end {
            return Err(CommandError::InvalidCutPoint(format!(
                "Cut point {:?} is outside clip range [{:?}, {:?})",
                self.cut_point, tl_start, tl_end
            )));
        }

        self.original_timeline_duration = Some(orig_clip.timeline_range.duration);
        self.original_source_duration = Some(orig_clip.source_range.duration);

        let first_tl_duration = RationalTime::new(cut_rescaled.value - tl_start.value, tl_start.timescale);
        let second_tl_duration = RationalTime::new(tl_end.value - cut_rescaled.value, tl_start.timescale);

        let src_scale = orig_clip.source_range.start.timescale;
        let speed_ratio = orig_clip.speed.abs();
        let first_src_ticks = (first_tl_duration.value as f64 * speed_ratio).round() as i64;
        let second_src_ticks = orig_clip.source_range.duration.value - first_src_ticks;

        let mut second_clip = orig_clip.clone();
        let new_id = Uuid::new_v4();
        self.created_clip_id = Some(new_id);
        second_clip.id = new_id;
        second_clip.name = format!("{} (Part 2)", orig_clip.name);

        // 更新前半段
        orig_clip.timeline_range.duration = first_tl_duration;
        orig_clip.source_range.duration = RationalTime::new(first_src_ticks, src_scale);

        // 设置后半段
        second_clip.timeline_range = TimeRange::new(cut_rescaled, second_tl_duration);
        if orig_clip.speed >= 0.0 {
            let second_src_start = RationalTime::new(orig_clip.source_range.start.value + first_src_ticks, src_scale);
            second_clip.source_range = TimeRange::new(second_src_start, RationalTime::new(second_src_ticks, src_scale));
        } else {
            // 倒放时
            second_clip.source_range = TimeRange::new(orig_clip.source_range.start, RationalTime::new(second_src_ticks, src_scale));
            orig_clip.source_range.start = RationalTime::new(orig_clip.source_range.start.value + second_src_ticks, src_scale);
        }

        track.clips.insert(clip_idx + 1, second_clip);
        Ok(())
    }

    fn undo(&mut self, sequence: &mut Sequence) -> Result<(), CommandError> {
        let (track, clip_idx) = sequence
            .find_clip_mut(self.target_clip_id)
            .ok_or(CommandError::ClipNotFound(self.target_clip_id))?;

        if let Some(new_id) = self.created_clip_id {
            if let Some(pos) = track.clips.iter().position(|c| c.id == new_id) {
                track.clips.remove(pos);
            }
        }

        let orig_clip = &mut track.clips[clip_idx];
        if let Some(tl_dur) = self.original_timeline_duration {
            orig_clip.timeline_range.duration = tl_dur;
        }
        if let Some(src_dur) = self.original_source_duration {
            orig_clip.source_range.duration = src_dur;
        }

        Ok(())
    }

    fn description(&self) -> &'static str {
        "分割片段"
    }
}
