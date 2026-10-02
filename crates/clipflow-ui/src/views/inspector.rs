pub struct MasterVuMeter {
    left_channel_db: f32,
    right_channel_db: f32,
}

impl Default for MasterVuMeter {
    fn default() -> Self {
        Self::new()
    }
}

impl MasterVuMeter {
    pub fn new() -> Self {
        Self {
            left_channel_db: -60.0,
            right_channel_db: -60.0,
        }
    }

    pub fn left_db(&self) -> f32 {
        self.left_channel_db
    }

    pub fn right_db(&self) -> f32 {
        self.right_channel_db
    }

    pub fn set_levels(&mut self, left: f32, right: f32) {
        self.left_channel_db = left.clamp(-60.0, 12.0);
        self.right_channel_db = right.clamp(-60.0, 12.0);
    }
}

pub struct InspectorPanel {
    vu_meter: MasterVuMeter,
    active_tab: String,
}

impl Default for InspectorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl InspectorPanel {
    pub fn new() -> Self {
        Self {
            vu_meter: MasterVuMeter::new(),
            active_tab: "Properties".to_string(),
        }
    }

    pub fn active_tab(&self) -> &str {
        &self.active_tab
    }

    pub fn set_active_tab(&mut self, tab: impl Into<String>) {
        self.active_tab = tab.into();
    }

    pub fn vu_meter(&self) -> &MasterVuMeter {
        &self.vu_meter
    }

    pub fn vu_meter_mut(&mut self) -> &mut MasterVuMeter {
        &mut self.vu_meter
    }
}
