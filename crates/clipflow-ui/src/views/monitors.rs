pub struct DualMonitorView {
    source_timecode: String,
    program_timecode: String,
    source_in_point: Option<String>,
    source_out_point: Option<String>,
    program_in_point: Option<String>,
    program_out_point: Option<String>,
    quality_preset: String,
    zoom_preset: String,
}

impl Default for DualMonitorView {
    fn default() -> Self {
        Self::new()
    }
}

impl DualMonitorView {
    pub fn new() -> Self {
        Self {
            source_timecode: "00:00:00:00".to_string(),
            program_timecode: "00:00:00:00".to_string(),
            source_in_point: None,
            source_out_point: None,
            program_in_point: None,
            program_out_point: None,
            quality_preset: "Full".to_string(),
            zoom_preset: "Fit".to_string(),
        }
    }

    pub fn source_timecode(&self) -> &str {
        &self.source_timecode
    }

    pub fn set_source_timecode(&mut self, tc: impl Into<String>) {
        self.source_timecode = tc.into();
    }

    pub fn program_timecode(&self) -> &str {
        &self.program_timecode
    }

    pub fn set_program_timecode(&mut self, tc: impl Into<String>) {
        self.program_timecode = tc.into();
    }

    pub fn program_in_point(&self) -> Option<&str> {
        self.program_in_point.as_deref()
    }

    pub fn set_program_in_point(&mut self, pt: impl Into<String>) {
        self.program_in_point = Some(pt.into());
    }

    pub fn program_out_point(&self) -> Option<&str> {
        self.program_out_point.as_deref()
    }

    pub fn set_program_out_point(&mut self, pt: impl Into<String>) {
        self.program_out_point = Some(pt.into());
    }

    pub fn source_in_point(&self) -> Option<&str> {
        self.source_in_point.as_deref()
    }

    pub fn set_source_in_point(&mut self, pt: impl Into<String>) {
        self.source_in_point = Some(pt.into());
    }

    pub fn source_out_point(&self) -> Option<&str> {
        self.source_out_point.as_deref()
    }

    pub fn set_source_out_point(&mut self, pt: impl Into<String>) {
        self.source_out_point = Some(pt.into());
    }

    pub fn quality_preset(&self) -> &str {
        &self.quality_preset
    }

    pub fn set_quality_preset(&mut self, q: impl Into<String>) {
        self.quality_preset = q.into();
    }

    pub fn zoom_preset(&self) -> &str {
        &self.zoom_preset
    }

    pub fn set_zoom_preset(&mut self, z: impl Into<String>) {
        self.zoom_preset = z.into();
    }
}
