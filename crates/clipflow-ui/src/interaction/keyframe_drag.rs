use uuid::Uuid;

/// 关键帧微调拖拽锁定上下文
///
/// 视觉几何 (6x6px 菱形) 与逻辑拾取判定区 (14x14px AABB，半边长 7px) 解耦，
/// 一旦按压命中即锁定独占焦点，防止高速拖拽滑脱。
#[derive(Debug, Clone, Default)]
pub struct DragLockContext {
    locked_kf: Option<Uuid>,
    initial_value: f32,
}

impl DragLockContext {
    pub fn new() -> Self {
        Self {
            locked_kf: None,
            initial_value: 0.0,
        }
    }

    /// 判定点是否落在关键帧的扩展拾取区域内 (AABB 检测)
    pub fn is_hit(center: (f32, f32), point: (f32, f32), half_size: f32) -> bool {
        (point.0 - center.0).abs() <= half_size && (point.1 - center.1).abs() <= half_size
    }

    pub fn start_drag(&mut self, id: Uuid, initial_val: f32) {
        self.locked_kf = Some(id);
        self.initial_value = initial_val;
    }

    pub fn end_drag(&mut self) {
        self.locked_kf = None;
        self.initial_value = 0.0;
    }

    pub fn is_dragging(&self) -> bool {
        self.locked_kf.is_some()
    }

    pub fn locked_keyframe_id(&self) -> Option<Uuid> {
        self.locked_kf
    }

    pub fn initial_value(&self) -> f32 {
        self.initial_value
    }
}
