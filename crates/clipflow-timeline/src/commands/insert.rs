use crate::commands::{CommandError, TimelineCommand};
use crate::models::clip::Clip;
use crate::models::sequence::Sequence;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct InsertClipCommand {
    pub target_track_id: Uuid,
    pub clip_to_insert: Clip,
    pub inserted_index: Option<usize>,
}

impl InsertClipCommand {
    pub fn new(target_track_id: Uuid, clip: Clip) -> Self {
        Self {
            target_track_id,
            clip_to_insert: clip,
            inserted_index: None,
        }
    }
}

impl TimelineCommand for InsertClipCommand {
    fn execute(&mut self, sequence: &mut Sequence) -> Result<(), CommandError> {
        let track = sequence
            .find_track_mut(self.target_track_id)
            .ok_or(CommandError::TrackNotFound(self.target_track_id))?;

        // 插入并按 timeline_range.start 升序保持有序
        let insert_pos = track
            .clips
            .partition_point(|c| c.timeline_range.start <= self.clip_to_insert.timeline_range.start);

        track.clips.insert(insert_pos, self.clip_to_insert.clone());
        self.inserted_index = Some(insert_pos);
        Ok(())
    }

    fn undo(&mut self, sequence: &mut Sequence) -> Result<(), CommandError> {
        let track = sequence
            .find_track_mut(self.target_track_id)
            .ok_or(CommandError::TrackNotFound(self.target_track_id))?;

        if let Some(pos) = track.clips.iter().position(|c| c.id == self.clip_to_insert.id) {
            track.clips.remove(pos);
        }
        Ok(())
    }

    fn description(&self) -> &'static str {
        "插入片段"
    }
}
