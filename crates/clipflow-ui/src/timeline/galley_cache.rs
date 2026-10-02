use std::collections::HashMap;

pub struct SubtitleGalleyCache {
    capacity: usize,
    cache: HashMap<(String, u32), (f32, f32)>, // 尺寸缓存 (width, height)
    access_order: Vec<(String, u32)>,
}

impl SubtitleGalleyCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: HashMap::with_capacity(capacity),
            access_order: Vec::with_capacity(capacity),
        }
    }

    fn key(text: &str, font_size: f32) -> (String, u32) {
        (text.to_string(), (font_size * 10.0) as u32)
    }

    pub fn contains(&self, text: &str, font_size: f32) -> bool {
        self.cache.contains_key(&Self::key(text, font_size))
    }

    pub fn get(&mut self, text: &str, font_size: f32) -> Option<(f32, f32)> {
        let k = Self::key(text, font_size);
        if let Some(&dims) = self.cache.get(&k) {
            // 更新 LRU
            if let Some(pos) = self.access_order.iter().position(|item| item == &k) {
                self.access_order.remove(pos);
            }
            self.access_order.push(k);
            Some(dims)
        } else {
            None
        }
    }

    pub fn insert(&mut self, text: &str, font_size: f32, dims: (f32, f32)) {
        let k = Self::key(text, font_size);
        if self.cache.len() >= self.capacity {
            if let Some(oldest) = self.access_order.first().cloned() {
                self.access_order.remove(0);
                self.cache.remove(&oldest);
            }
        }
        self.cache.insert(k.clone(), dims);
        self.access_order.push(k);
    }
}
