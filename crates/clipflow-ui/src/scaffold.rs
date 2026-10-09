use egui::Ui;

pub struct EditScaffold {
    left_width: f32,
    right_width: f32,
}

impl Default for EditScaffold {
    fn default() -> Self {
        Self {
            left_width: 320.0,
            right_width: 200.0,
        }
    }
}

impl EditScaffold {
    pub fn left_panel_width(&self) -> f32 {
        self.left_width
    }

    pub fn right_panel_width(&self) -> f32 {
        self.right_width
    }

    /// 渲染 PR 剪辑工作台骨架布局
    pub fn ui(&mut self, ui: &mut Ui, left_content: impl FnOnce(&mut Ui), center_content: impl FnOnce(&mut Ui), right_content: impl FnOnce(&mut Ui)) {
        // 左侧栏：素材池与源监视器区域
        egui::Panel::left("edit_left_panel")
            .default_size(self.left_width)
            .resizable(true)
            .show(ui, |ui| {
                left_content(ui);
            });

        // 右侧栏：属性与效果控件区域
        egui::Panel::right("edit_right_panel")
            .default_size(self.right_width)
            .resizable(true)
            .show(ui, |ui| {
                right_content(ui);
            });

        // 中央区域：主合成监视器与时间轴
        egui::CentralPanel::default().show(ui, |ui| {
            center_content(ui);
        });
    }
}
