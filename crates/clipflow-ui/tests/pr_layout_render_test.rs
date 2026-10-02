use clipflow_ui::views::{
    DualMonitorView, EditWorkspaceView, ProjectPanel, ResizableSplitter, ViewMode,
};

#[test]
fn test_pr_layout_split_proportions() {
    let workspace = EditWorkspaceView::new();
    assert_eq!(workspace.upper_height_ratio(), 0.58);
    assert_eq!(workspace.zone1_width_ratio(), 0.25);
    assert_eq!(workspace.zone2_width_ratio(), 0.50);
    assert_eq!(workspace.zone3_width_ratio(), 0.25);
}

#[test]
fn test_dual_monitor_tab_and_transport_controls() {
    let mut monitors = DualMonitorView::new();
    assert_eq!(monitors.source_timecode(), "00:00:00:00");
    assert_eq!(monitors.program_timecode(), "00:00:00:00");

    monitors.set_source_in_point("00:00:05:00");
    monitors.set_source_out_point("00:00:15:00");
    assert_eq!(monitors.source_in_point(), Some("00:00:05:00"));
    assert_eq!(monitors.source_out_point(), Some("00:00:15:00"));

    // 缩放模式与画质档位切换
    monitors.set_quality_preset("1/4");
    assert_eq!(monitors.quality_preset(), "1/4");
}

#[test]
fn test_project_panel_grid_and_list_mode_switch() {
    let mut panel = ProjectPanel::new();
    assert_eq!(panel.view_mode(), ViewMode::Grid);

    panel.set_view_mode(ViewMode::List);
    assert_eq!(panel.view_mode(), ViewMode::List);

    panel.set_search_query("footage");
    assert_eq!(panel.search_query(), "footage");
}

#[test]
fn test_splitter_drag_constraints() {
    let mut splitter = ResizableSplitter::new(0.58, 0.30, 0.70);
    assert_eq!(splitter.ratio(), 0.58);

    // 向上拖拽越界箝位至 0.30
    splitter.set_ratio(0.10);
    assert_eq!(splitter.ratio(), 0.30);

    // 向下拖拽越界箝位至 0.70
    splitter.set_ratio(0.95);
    assert_eq!(splitter.ratio(), 0.70);

    // 正常区间内自由设置
    splitter.set_ratio(0.50);
    assert_eq!(splitter.ratio(), 0.50);
}
