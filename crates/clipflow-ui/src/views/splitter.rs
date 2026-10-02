pub struct ResizableSplitter {
    ratio: f32,
    min_ratio: f32,
    max_ratio: f32,
}

impl ResizableSplitter {
    pub fn new(initial_ratio: f32, min_ratio: f32, max_ratio: f32) -> Self {
        let clamped = initial_ratio.clamp(min_ratio, max_ratio);
        Self {
            ratio: clamped,
            min_ratio,
            max_ratio,
        }
    }

    pub fn ratio(&self) -> f32 {
        self.ratio
    }

    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(self.min_ratio, self.max_ratio);
    }
}
