use crate::commands::{CommandError, TimelineCommand};
use crate::models::clip::Clip;
use crate::models::sequence::Sequence;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DeleteClipCommand {
    pub target_clip_id: Uuid,
    deleted_clip: Option<Clip>,
    track_id: Option<Uuid>,
    index: Option<usize>,
}

impl DeleteClipCommand {
    pub fn new(target_clip_id: Uuid) -> Self {
        Self {
            target_clip_id,
            deleted_clip: None,
            track_id: None,
            index: None,
        }
    }
}

impl TimelineCommand for DeleteClipCommand {
    fn execute(&mut self, sequence: &mut Sequence) -> Result<(), CommandError> {
        let (track, clip_idx) = sequence
            .find_clip_mut(self.target_clip_id)
            .ok_or(CommandError::ClipNotFound(self.target_clip_id))?;

        let track_id = track.id;
        let clip = track.clips.remove(clip_idx);

        self.deleted_clip = Some(clip);
        self.track_id = Some(track_id);
        self.index = Some(clip_idx);
        Ok(())
    }

    fn undo(&mut self, sequence: &mut Sequence) -> Result<(), CommandError> {
        let clip = self.deleted_clip.take().ok_or(CommandError::ExecutionFailed(
            "No clip to restore in DeleteClipCommand".to_string(),
        ))?;
        let track_id = self.track_id.ok_or(CommandError::ExecutionFailed(
            "No track ID in DeleteClipCommand".to_string(),
        ))?;
        let idx = self.index.unwrap_or(0);

        let track = sequence
            .find_track_mut(track_id)
            .ok_or(CommandError::TrackNotFound(track_id))?;

        let safe_idx = idx.min(track.clips.len());
        track.clips.insert(safe_idx, clip);
        Ok(())
    }

    fn description(&self) -> &'static str {
        "删除片段"
    }
}
