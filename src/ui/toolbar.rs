use crate::editor::tool_manager::ToolKind;

pub struct ToolbarState {
    pub active_tool: ToolKind,
}

impl ToolbarState {
    pub fn new() -> Self {
        Self {
            active_tool: ToolKind::Brush,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Tools");
        ui.add_space(4.0);

        for (kind, label, shortcut) in [
            (ToolKind::Brush, "Brush", "B"),
            (ToolKind::Pen, "Pen", "P"),
            (ToolKind::Select, "Select", "S"),
            (ToolKind::Label, "Label", "L"),
            (ToolKind::Eraser, "Eraser", "E"),
            (ToolKind::Eyedropper, "Eyedropper", "I"),
        ] {
            let selected = self.active_tool == kind;
            if ui
                .selectable_label(selected, format!("{} ({})", label, shortcut))
                .clicked()
            {
                self.active_tool = kind;
            }
        }
    }

    /// Handle keyboard shortcut and return true if a tool was switched.
    pub fn handle_key(&mut self, key: egui::Key) -> bool {
        let new_tool = match key {
            egui::Key::B => Some(ToolKind::Brush),
            egui::Key::P => Some(ToolKind::Pen),
            egui::Key::S => Some(ToolKind::Select),
            egui::Key::L => Some(ToolKind::Label),
            egui::Key::E => Some(ToolKind::Eraser),
            egui::Key::I => Some(ToolKind::Eyedropper),
            _ => None,
        };
        if let Some(tool) = new_tool {
            self.active_tool = tool;
            true
        } else {
            false
        }
    }
}
