use clipflow_timeline::models::TrackKind;

pub struct TrackHeaderWidget {
    pub name: String,
    pub kind: TrackKind,
    pub locked: bool,
    pub visible: bool,
    pub mute: bool,
    pub solo: bool,
    pub height: f32,
}

impl TrackHeaderWidget {
    pub fn new(name: impl Into<String>, kind: TrackKind) -> Self {
        let height = match kind {
            TrackKind::Audio => 72.0,
            _ => 64.0,
        };
        Self {
            name: name.into(),
            kind,
            locked: false,
            visible: true,
            mute: false,
            solo: false,
            height,
        }
    }

    /// 返回该轨道对应的 Neutral Modern 轨道底板与边框高亮色
    pub fn color_tokens(&self) -> (&'static str, &'static str) {
        match self.kind {
            TrackKind::Subtitle => ("#271F12", "#63471C"),  // 暗金底 / 金棕边框
            TrackKind::HyperFrames => ("#22142D", "#6B21A8"), // 深曜紫
            TrackKind::Video => ("#151E2E", "#2B4570"),      // 沉稳蓝
            TrackKind::Audio => ("#122119", "#1E4A32"),      // 暗墨绿
        }
    }
}
