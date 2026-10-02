use crate::commands::{CommandError, TimelineCommand};
use crate::models::sequence::Sequence;

pub struct CompoundCommand {
    commands: Vec<Box<dyn TimelineCommand>>,
    description: &'static str,
}

impl CompoundCommand {
    pub fn new(commands: Vec<Box<dyn TimelineCommand>>, description: &'static str) -> Self {
        Self {
            commands,
            description,
        }
    }
}

impl TimelineCommand for CompoundCommand {
    fn execute(&mut self, sequence: &mut Sequence) -> Result<(), CommandError> {
        let mut executed = 0;
        for cmd in &mut self.commands {
            if let Err(e) = cmd.execute(sequence) {
                // 回退已经执行的子命令以保证原子性
                for rolled_back_cmd in self.commands[..executed].iter_mut().rev() {
                    let _ = rolled_back_cmd.undo(sequence);
                }
                return Err(e);
            }
            executed += 1;
        }
        Ok(())
    }

    fn undo(&mut self, sequence: &mut Sequence) -> Result<(), CommandError> {
        // 逆序回退
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo(sequence)?;
        }
        Ok(())
    }

    fn description(&self) -> &'static str {
        self.description
    }
}
