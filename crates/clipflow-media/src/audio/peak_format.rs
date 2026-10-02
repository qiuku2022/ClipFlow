#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeakFileHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub channels: u16,
    pub sample_rate: u32,
    pub window_size: u32,
}

impl PeakFileHeader {
    pub fn new(sample_rate: u32, channels: u16, window_size: u32) -> Self {
        Self {
            magic: *b"CFPK",
            version: 1,
            channels,
            sample_rate,
            window_size,
        }
    }

    pub fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&self.magic);
        bytes[4..6].copy_from_slice(&self.version.to_le_bytes());
        bytes[6..8].copy_from_slice(&self.channels.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.sample_rate.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.window_size.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 16 || &bytes[0..4] != b"CFPK" {
            return None;
        }
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        let channels = u16::from_le_bytes([bytes[6], bytes[7]]);
        let sample_rate = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let window_size = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        Some(Self {
            magic: *b"CFPK",
            version,
            channels,
            sample_rate,
            window_size,
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeakRecord {
    pub min_sample: i16,
    pub max_sample: i16,
    pub rms_energy: u16,
}

impl PeakRecord {
    pub fn to_bytes(&self) -> [u8; 6] {
        let mut bytes = [0u8; 6];
        bytes[0..2].copy_from_slice(&self.min_sample.to_le_bytes());
        bytes[2..4].copy_from_slice(&self.max_sample.to_le_bytes());
        bytes[4..6].copy_from_slice(&self.rms_energy.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 6 {
            return None;
        }
        let min_sample = i16::from_le_bytes([bytes[0], bytes[1]]);
        let max_sample = i16::from_le_bytes([bytes[2], bytes[3]]);
        let rms_energy = u16::from_le_bytes([bytes[4], bytes[5]]);
        Some(Self {
            min_sample,
            max_sample,
            rms_energy,
        })
    }

    pub fn read_records(bytes: &[u8]) -> Option<Vec<Self>> {
        let count = bytes.len() / 6;
        let mut records = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * 6;
            if let Some(r) = Self::from_bytes(&bytes[offset..offset + 6]) {
                records.push(r);
            }
        }
        Some(records)
    }
}
