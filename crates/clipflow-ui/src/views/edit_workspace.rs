use crate::views::inspector::InspectorPanel;
use crate::views::monitors::DualMonitorView;
use crate::views::project_panel::ProjectPanel;
use crate::views::splitter::ResizableSplitter;

pub struct EditWorkspaceView {
    splitter: ResizableSplitter,
    project_panel: ProjectPanel,
    monitors: DualMonitorView,
    inspector: InspectorPanel,
    zone1_width_ratio: f32,
    zone2_width_ratio: f32,
    zone3_width_ratio: f32,
}

impl Default for EditWorkspaceView {
    fn default() -> Self {
        Self::new()
    }
}

impl EditWorkspaceView {
    pub fn new() -> Self {
        Self {
            splitter: ResizableSplitter::new(0.58, 0.30, 0.70),
            project_panel: ProjectPanel::new(),
            monitors: DualMonitorView::new(),
            inspector: InspectorPanel::new(),
            zone1_width_ratio: 0.25,
            zone2_width_ratio: 0.50,
            zone3_width_ratio: 0.25,
        }
    }

    pub fn upper_height_ratio(&self) -> f32 {
        self.splitter.ratio()
    }

    pub fn zone1_width_ratio(&self) -> f32 {
        self.zone1_width_ratio
    }

    pub fn zone2_width_ratio(&self) -> f32 {
        self.zone2_width_ratio
    }

    pub fn zone3_width_ratio(&self) -> f32 {
        self.zone3_width_ratio
    }

    pub fn project_panel(&self) -> &ProjectPanel {
        &self.project_panel
    }

    pub fn monitors(&self) -> &DualMonitorView {
        &self.monitors
    }

    pub fn inspector(&self) -> &InspectorPanel {
        &self.inspector
    }

    pub fn splitter_mut(&mut self) -> &mut ResizableSplitter {
        &mut self.splitter
    }
}
