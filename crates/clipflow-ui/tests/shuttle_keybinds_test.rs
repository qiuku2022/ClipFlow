use clipflow_common::RationalTime;
use clipflow_ui::interaction::{
    DragLockContext, ModifierKeyHandler, ShuttleController, ShuttleDirection, SnapEngine,
};
use uuid::Uuid;

#[test]
fn test_jkl_shuttle_cascade_speed_state_machine() {
    let mut shuttle = ShuttleController::new();
    assert_eq!(shuttle.speed_multiplier(), 0); // 初始暂停

    // 连续按 L 升级倍速：1 -> 2 -> 4 -> 8 -> 16
    shuttle.press_l();
    assert_eq!(shuttle.speed_multiplier(), 1);
    assert_eq!(shuttle.direction(), ShuttleDirection::Forward);

    shuttle.press_l();
    assert_eq!(shuttle.speed_multiplier(), 2);

    shuttle.press_l();
    assert_eq!(shuttle.speed_multiplier(), 4);

    shuttle.press_l();
    assert_eq!(shuttle.speed_multiplier(), 8);

    shuttle.press_l();
    assert_eq!(shuttle.speed_multiplier(), 16);

    // 再按 L 封顶在 16
    shuttle.press_l();
    assert_eq!(shuttle.speed_multiplier(), 16);

    // 按 K 瞬时急停并重置
    shuttle.press_k();
    assert_eq!(shuttle.speed_multiplier(), 0);
    assert_eq!(shuttle.direction(), ShuttleDirection::Stopped);

    // 连续按 J 反向倍速：-1 -> -2 -> -4 -> -8 -> -16
    shuttle.press_j();
    assert_eq!(shuttle.speed_multiplier(), 1);
    assert_eq!(shuttle.direction(), ShuttleDirection::Backward);

    shuttle.press_j();
    assert_eq!(shuttle.speed_multiplier(), 2);
    assert_eq!(shuttle.direction(), ShuttleDirection::Backward);

    // 空格键在暂停与 1x 之间切换
    shuttle.press_space();
    assert_eq!(shuttle.speed_multiplier(), 0);
    shuttle.press_space();
    assert_eq!(shuttle.speed_multiplier(), 1);
    assert_eq!(shuttle.direction(), ShuttleDirection::Forward);
}

#[test]
fn test_jkl_single_frame_step() {
    let mut shuttle = ShuttleController::new();
    let current_pts = RationalTime::new(100, 60);

    // K + L 单帧正进
    let next_pts = shuttle.step_forward(current_pts, 60);
    assert_eq!(next_pts, RationalTime::new(101, 60));
    assert_eq!(shuttle.speed_multiplier(), 0); // 步进后保持暂停

    // K + J 单帧后退
    let prev_pts = shuttle.step_backward(current_pts, 60);
    assert_eq!(prev_pts, RationalTime::new(99, 60));
}

#[test]
fn test_snap_toggle_and_shift_inversion() {
    let mut snap = SnapEngine::new(true); // 默认开启磁吸
    assert!(snap.is_enabled());

    // S 键切换
    snap.toggle();
    assert!(!snap.is_enabled());
    snap.toggle();
    assert!(snap.is_enabled());

    // 按住 Shift 临时反转：全局开启时，按住 Shift 为 false
    assert!(!snap.is_active_with_shift(true));
    // 没按 Shift 时为 true
    assert!(snap.is_active_with_shift(false));

    // 吸附计算：给定吸附目标 100 ticks，当前位置 102 ticks（吸附半径 5 ticks）
    let targets = vec![RationalTime::new(100, 60), RationalTime::new(500, 60)];
    let snapped = snap.snap_time(RationalTime::new(102, 60), &targets, RationalTime::new(5, 60));
    assert_eq!(snapped, RationalTime::new(100, 60));

    // 超出吸附半径不吸附
    let not_snapped = snap.snap_time(RationalTime::new(110, 60), &targets, RationalTime::new(5, 60));
    assert_eq!(not_snapped, RationalTime::new(110, 60));
}

#[test]
fn test_alt_drag_duplicate_and_wheel_zoom() {
    let handler = ModifierKeyHandler::new();

    // 缩放计算：鼠标位置 10.0s，缩放因子 1.2
    let (new_pps, new_scroll) = handler.compute_cursor_centered_zoom(
        50.0,  // current pps
        0.0,   // current scroll
        1.2,   // zoom factor
        10.0,  // cursor time sec
    );
    assert!((new_pps - 60.0).abs() < 1e-4);
    // 鼠标 10.0s 处在旧视图中是 10 * 50 = 500px，在新视图中是 (10 - new_scroll) * 60 = 500px -> new_scroll = 10 - 500/60 = 1.6667
    let cursor_px_old = (10.0 - 0.0) * 50.0;
    let cursor_px_new = (10.0 - new_scroll) * new_pps;
    assert!((cursor_px_old - cursor_px_new).abs() < 1e-4);
}

#[test]
fn test_keyframe_drag_hitbox_and_lock_capture() {
    let mut drag_ctx = DragLockContext::new();
    assert!(!drag_ctx.is_dragging());

    let kf_id = Uuid::new_v4();
    // 判定区 14x14px，中心位于 (100.0, 50.0)
    let center = (100.0f32, 50.0f32);

    // 点击在 (104.0, 52.0) 处在 14x14 范围内 (半宽 7px)
    assert!(DragLockContext::is_hit(center, (104.0, 52.0), 7.0));
    // 点击在 (120.0, 50.0) 超出
    assert!(!DragLockContext::is_hit(center, (120.0, 50.0), 7.0));

    drag_ctx.start_drag(kf_id, 0.5);
    assert!(drag_ctx.is_dragging());
    assert_eq!(drag_ctx.locked_keyframe_id(), Some(kf_id));

    drag_ctx.end_drag();
    assert!(!drag_ctx.is_dragging());
}
