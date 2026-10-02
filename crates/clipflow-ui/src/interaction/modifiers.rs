/// 键盘修饰键与鼠标交互处理器
#[derive(Debug, Clone, Default)]
pub struct ModifierKeyHandler;

impl ModifierKeyHandler {
    pub fn new() -> Self {
        Self
    }

    /// 以光标时间点为中心计算缩放后的 pps 与新滚动位置
    ///
    /// 原理：
    /// 缩放前后，光标指向的绝对时间点在屏幕像素空间中的相对偏移保持恒定：
    /// `cursor_px = (cursor_time_sec - current_scroll) * current_pps`
    /// `new_scroll = cursor_time_sec - cursor_px / new_pps`
    pub fn compute_cursor_centered_zoom(
        &self,
        current_pps: f32,
        current_scroll: f32,
        zoom_factor: f32,
        cursor_time_sec: f32,
    ) -> (f32, f32) {
        let new_pps = current_pps * zoom_factor;
        let cursor_px = (cursor_time_sec - current_scroll) * current_pps;
        let new_scroll = cursor_time_sec - cursor_px / new_pps;
        (new_pps, new_scroll)
    }
}
