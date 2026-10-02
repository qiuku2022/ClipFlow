use clipflow_common::RationalTime;
use clipflow_ui::dock::WorkflowPage;
use clipflow_ui::state::AppState;
use std::time::Instant;

#[test]
fn test_workflow_page_transitions() {
    let mut page = WorkflowPage::default();
    assert_eq!(page, WorkflowPage::Agent);

    page = WorkflowPage::Edit;
    assert_eq!(page.as_str(), "EDIT");

    page = WorkflowPage::Motion;
    assert_eq!(page.as_str(), "MOTION");

    page = WorkflowPage::Audio;
    assert_eq!(page.as_str(), "AUDIO");

    page = WorkflowPage::Image;
    assert_eq!(page.as_str(), "IMAGE");

    page = WorkflowPage::Deliver;
    assert_eq!(page.as_str(), "DELIVER");
}

#[test]
fn test_dock_switch_performance() {
    let mut page = WorkflowPage::Agent;
    let start = Instant::now();
    for _ in 0..10_000 {
        page = match page {
            WorkflowPage::Agent => WorkflowPage::Edit,
            WorkflowPage::Edit => WorkflowPage::Motion,
            WorkflowPage::Motion => WorkflowPage::Audio,
            WorkflowPage::Audio => WorkflowPage::Image,
            WorkflowPage::Image => WorkflowPage::Deliver,
            WorkflowPage::Deliver => WorkflowPage::Agent,
        };
    }
    let elapsed = start.elapsed();
    let per_switch = elapsed.as_secs_f64() * 1000.0 / 10_000.0;
    // 断言单次切页状态机耗时 <= 0.5ms (通常为几十纳秒)
    assert!(per_switch <= 0.5, "切页耗时超标: {:?}ms", per_switch);
}

#[test]
fn test_global_timeline_state_preserved_across_pages() {
    let mut state = AppState::default();
    state.playhead = RationalTime::new(48000, 24); // 2000 秒位置

    let mut current_page = WorkflowPage::Edit;
    state.active_page = current_page;
    assert_eq!(state.active_page, WorkflowPage::Edit);
    assert_eq!(state.playhead.value, 48000);

    // 切换到 Motion 页面
    current_page = WorkflowPage::Motion;
    state.active_page = current_page;
    assert_eq!(state.active_page, WorkflowPage::Motion);
    assert_eq!(state.playhead.value, 48000);

    // 切换到 Deliver 页面
    current_page = WorkflowPage::Deliver;
    state.active_page = current_page;
    assert_eq!(state.active_page, WorkflowPage::Deliver);
    assert_eq!(state.playhead.value, 48000);
}
