use clipflow_common::{RationalTime, TimeRange};
use std::time::{Duration, Instant};

/// 刷新调度器三态门控
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepaintState {
    /// 绝对休眠态：暂停且无交互超过 150ms，挂起 OS 消息队列，0.0% CPU 占用
    Dormant,
    /// 音画播放态：以 60 FPS 节拍器触发重绘
    Playing,
    /// 即时交互态：用户正在拖拽或缩放，以满刷新率响应
    Interacting,
}

/// 门控调度器
#[derive(Debug)]
pub struct RepaintScheduler {
    state: RepaintState,
    last_interaction: Instant,
    dormant_threshold: Duration,
}

impl Default for RepaintScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl RepaintScheduler {
    pub fn new() -> Self {
        Self {
            state: RepaintState::Interacting,
            last_interaction: Instant::now(),
            dormant_threshold: Duration::from_millis(150),
        }
    }

    pub fn state(&self) -> RepaintState {
        self.state
    }

    /// 标记用户发生输入交互（鼠标移动、按键、滚轮）
    pub fn on_user_interaction(&mut self) {
        self.last_interaction = Instant::now();
        self.state = RepaintState::Interacting;
    }

    /// 更新门控状态机，返回当前应处于的状态
    pub fn update_gating(&mut self, is_playing: bool) -> RepaintState {
        if is_playing {
            self.state = RepaintState::Playing;
            return self.state;
        }

        if self.last_interaction.elapsed() >= self.dormant_threshold {
            self.state = RepaintState::Dormant;
        } else {
            self.state = RepaintState::Interacting;
        }

        self.state
    }

    /// 仅供单元测试：人为推移时间
    pub fn advance_time_for_test(&mut self, duration: Duration) {
        if let Some(earlier) = self.last_interaction.checked_sub(duration) {
            self.last_interaction = earlier;
        }
    }
}

/// 全局应用程序状态根
#[derive(Debug)]
pub struct AppState {
    pub playhead: RationalTime,
    pub is_playing: bool,
    pub zoom_level: f32,
    pub visible_time_range: TimeRange,
    pub scheduler: RepaintScheduler,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            playhead: RationalTime::ZERO,
            is_playing: false,
            zoom_level: 1.0,
            visible_time_range: TimeRange::new(
                RationalTime::ZERO,
                RationalTime::new(60000, 1000), // 初始可见 60 秒
            ),
            scheduler: RepaintScheduler::new(),
        }
    }
}
