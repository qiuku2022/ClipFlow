#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Grid,
    List,
}

pub struct ProjectPanel {
    view_mode: ViewMode,
    search_query: String,
    selected_items: Vec<String>,
}

impl Default for ProjectPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectPanel {
    pub fn new() -> Self {
        Self {
            view_mode: ViewMode::Grid,
            search_query: String::new(),
            selected_items: Vec::new(),
        }
    }

    pub fn view_mode(&self) -> ViewMode {
        self.view_mode
    }

    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn set_search_query(&mut self, query: impl Into<String>) {
        self.search_query = query.into();
    }

    pub fn selected_count(&self) -> usize {
        self.selected_items.len()
    }
}
