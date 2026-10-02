use crate::commands::{CommandError, TimelineCommand};
use crate::models::sequence::Sequence;

pub struct TimelineHistory {
    undo_stack: Vec<Box<dyn TimelineCommand>>,
    redo_stack: Vec<Box<dyn TimelineCommand>>,
    max_depth: usize,
}

impl TimelineHistory {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth,
        }
    }

    pub fn execute(
        &mut self,
        mut cmd: Box<dyn TimelineCommand>,
        seq: &mut Sequence,
    ) -> Result<(), CommandError> {
        cmd.execute(seq)?;
        self.undo_stack.push(cmd);
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        Ok(())
    }

    pub fn undo(&mut self, seq: &mut Sequence) -> Result<bool, CommandError> {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.undo(seq)?;
            self.redo_stack.push(cmd);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn redo(&mut self, seq: &mut Sequence) -> Result<bool, CommandError> {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute(seq)?;
            self.undo_stack.push(cmd);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_len(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_len(&self) -> usize {
        self.redo_stack.len()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}
