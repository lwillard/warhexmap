use crate::editor::brush_tool::PaintMode;
use crate::model::terrain_types::{Decorator, PathType};

pub struct PaletteState {
    pub paint_mode: PaintMode,
    pub selected_decorator: Decorator,
    pub selected_path_type: PathType,
}

impl PaletteState {
    pub fn new() -> Self {
        Self {
            paint_mode: PaintMode::Elevation,
            selected_decorator: Decorator::ALL[0],
            selected_path_type: PathType::ALL[0],
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Palette");
        ui.add_space(4.0);

        // Paint mode radio buttons
        ui.label("Paint Mode");
        ui.radio_value(&mut self.paint_mode, PaintMode::Elevation, "Elevation");
        ui.radio_value(&mut self.paint_mode, PaintMode::Climate, "Climate");
        ui.radio_value(&mut self.paint_mode, PaintMode::Decorator, "Decorator");
        ui.add_space(8.0);

        // Decorator selection
        ui.collapsing("Decorators", |ui| {
            for &dec in Decorator::ALL.iter() {
                ui.radio_value(&mut self.selected_decorator, dec, dec.label());
            }
        });
        ui.add_space(4.0);

        // Path type selection
        ui.collapsing("Path Types", |ui| {
            for &pt in PathType::ALL.iter() {
                ui.radio_value(&mut self.selected_path_type, pt, pt.label());
            }
        });
    }
}
