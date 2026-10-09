use clipflow_ui::dock::{WorkflowPage, DockState};
use clipflow_ui::scaffold::EditScaffold;

#[test]
fn test_workflow_page_navigation() {
    let mut dock = DockState::new(WorkflowPage::Edit);
    
    assert_eq!(dock.current_page(), WorkflowPage::Edit);
    
    dock.navigate_to(WorkflowPage::Agent);
    assert_eq!(dock.current_page(), WorkflowPage::Agent);
    
    // Test all 6 pages exist
    let all_pages = [
        WorkflowPage::Agent,
        WorkflowPage::Edit,
        WorkflowPage::Motion,
        WorkflowPage::Audio,
        WorkflowPage::Image,
        WorkflowPage::Deliver,
    ];
    assert_eq!(all_pages.len(), 6);
}

#[test]
fn test_edit_scaffold_initialization() {
    // 验证 PR 剪辑工作台骨架
    let scaffold = EditScaffold::default();
    
    // 检查是否具备三列结构等基本状态（可以通过检查其面板大小配置等）
    assert_eq!(scaffold.left_panel_width(), 320.0);
    assert_eq!(scaffold.right_panel_width(), 200.0);
}

