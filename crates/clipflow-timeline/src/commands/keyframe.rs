use crate::commands::{CommandError, TimelineCommand};
use crate::models::sequence::Sequence;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ModifyKeyframeCommand {
    pub clip_id: Uuid,
    pub property_name: String,
    pub keyframe_index: usize,
    pub old_value: f32,
    pub new_value: f32,
}

impl ModifyKeyframeCommand {
    pub fn new(
        clip_id: Uuid,
        property_name: impl Into<String>,
        keyframe_index: usize,
        old_value: f32,
        new_value: f32,
    ) -> Self {
        Self {
            clip_id,
            property_name: property_name.into(),
            keyframe_index,
            old_value,
            new_value,
        }
    }
}

impl TimelineCommand for ModifyKeyframeCommand {
    fn execute(&mut self, _sequence: &mut Sequence) -> Result<(), CommandError> {
        // 关键帧修改
        Ok(())
    }

    fn undo(&mut self, _sequence: &mut Sequence) -> Result<(), CommandError> {
        // 关键帧回退
        Ok(())
    }

    fn description(&self) -> &'static str {
        "调整关键帧参数"
    }
}
