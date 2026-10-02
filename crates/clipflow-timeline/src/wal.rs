use clipflow_common::RationalTime;
use serde::{Deserialize, Serialize};
use std::io::Result as IoResult;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WalOperation {
    Split {
        clip_id: Uuid,
        cut_point: RationalTime,
    },
    Delete {
        clip_id: Uuid,
    },
    RippleDelete {
        clip_id: Uuid,
    },
    Insert {
        track_id: Uuid,
        clip_id: Uuid,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalEntry {
    pub sequence_number: u64,
    pub timestamp_ms: i64,
    pub op: WalOperation,
}

pub struct TimelineWal {
    entries: Vec<WalEntry>,
    next_seq: u64,
}

impl TimelineWal {
    pub fn new_in_memory() -> Self {
        Self {
            entries: Vec::new(),
            next_seq: 1,
        }
    }

    pub fn append_split(&mut self, clip_id: Uuid, cut_point: RationalTime) -> IoResult<()> {
        let entry = WalEntry {
            sequence_number: self.next_seq,
            timestamp_ms: 0,
            op: WalOperation::Split { clip_id, cut_point },
        };
        self.next_seq += 1;
        self.entries.push(entry);
        Ok(())
    }

    pub fn append_delete(&mut self, clip_id: Uuid) -> IoResult<()> {
        let entry = WalEntry {
            sequence_number: self.next_seq,
            timestamp_ms: 0,
            op: WalOperation::Delete { clip_id },
        };
        self.next_seq += 1;
        self.entries.push(entry);
        Ok(())
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn read_entries(&self) -> IoResult<Vec<WalEntry>> {
        Ok(self.entries.clone())
    }
}
