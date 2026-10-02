use crate::commands::{CommandError, TimelineCommand};
use crate::models::clip::Clip;
use crate::models::sequence::Sequence;
use clipflow_common::RationalTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RippleDeleteCommand {
    pub target_clip_id: Uuid,
    pub affected_track_ids: Option<Vec<Uuid>>,
    deleted_clip: Option<Clip>,
    deleted_track_id: Option<Uuid>,
    deleted_index: Option<usize>,
    shifted_clips: Vec<(Uuid, RationalTime)>, // (ClipId, 原始开始时间)
}

impl RippleDeleteCommand {
    pub fn new(target_clip_id: Uuid, affected_track_ids: Option<Vec<Uuid>>) -> Self {
        Self {
            target_clip_id,
            affected_track_ids,
            deleted_clip: None,
            deleted_track_id: None,
            deleted_index: None,
            shifted_clips: Vec::new(),
        }
    }
}

impl TimelineCommand for RippleDeleteCommand {
    fn execute(&mut self, sequence: &mut Sequence) -> Result<(), CommandError> {
        let (track, clip_idx) = sequence
            .find_clip_mut(self.target_clip_id)
            .ok_or(CommandError::ClipNotFound(self.target_clip_id))?;

        let track_id = track.id;
        let clip = track.clips.remove(clip_idx);
        let shift_duration = clip.timeline_range.duration;
        let delete_start = clip.timeline_range.start;

        self.deleted_clip = Some(clip);
        self.deleted_track_id = Some(track_id);
        self.deleted_index = Some(clip_idx);
        self.shifted_clips.clear();

        // 确定需要联动的轨道集合
        let tracks_to_shift: Vec<Uuid> = if let Some(ref allowed) = self.affected_track_ids {
            allowed.clone()
        } else {
            // 默认策略：仅该轨道以及所有未锁定的主画/主音/字幕轨（不含显式 BGM 轨）
            sequence
                .tracks
                .iter()
                .filter(|t| !t.locked && !t.name.contains("BGM"))
                .map(|t| t.id)
                .collect()
        };

        for t in &mut sequence.tracks {
            if tracks_to_shift.contains(&t.id) {
                for c in &mut t.clips {
                    if c.timeline_range.start >= delete_start {
                        self.shifted_clips.push((c.id, c.timeline_range.start));
                        let dur_rescaled = shift_duration.rescaled_to(c.timeline_range.start.timescale);
                        c.timeline_range.start = RationalTime::new(
                            c.timeline_range.start.value - dur_rescaled.value,
                            c.timeline_range.start.timescale,
                        );
                    }
                }
            }
        }

        Ok(())
    }

    fn undo(&mut self, sequence: &mut Sequence) -> Result<(), CommandError> {
        let deleted_clip = self.deleted_clip.take().ok_or(CommandError::ExecutionFailed(
            "No clip to restore in RippleDeleteCommand".to_string(),
        ))?;
        let track_id = self.deleted_track_id.ok_or(CommandError::ExecutionFailed(
            "No track ID in RippleDeleteCommand".to_string(),
        ))?;
        let insert_idx = self.deleted_index.unwrap_or(0);

        // 恢复平移的切片
        for &(clip_id, original_start) in &self.shifted_clips {
            for t in &mut sequence.tracks {
                if let Some(c) = t.clips.iter_mut().find(|c| c.id == clip_id) {
                    c.timeline_range.start = original_start;
                }
            }
        }

        // 恢复被删除的切片
        let track = sequence
            .find_track_mut(track_id)
            .ok_or(CommandError::TrackNotFound(track_id))?;
        let safe_idx = insert_idx.min(track.clips.len());
        track.clips.insert(safe_idx, deleted_clip);

        Ok(())
    }

    fn description(&self) -> &'static str {
        "波纹删除"
    }
}
