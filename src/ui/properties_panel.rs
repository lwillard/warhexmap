use crate::model::hex_cell::HexCell;
use crate::model::terrain_types::{Climate, Elevation};

pub struct PropertiesState {
    pub hovered_hex: Option<HexCell>,
    pub selected_elevation: Elevation,
    pub selected_climate: Climate,
    pub brush_radius: i32,
}

impl PropertiesState {
    pub fn new() -> Self {
        Self {
            hovered_hex: None,
            selected_elevation: Elevation::ALL[0],
            selected_climate: Climate::ALL[0],
            brush_radius: 1,
        }
    }

    /// Draw the properties panel. Returns true if brush settings changed this frame.
    pub fn show(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        // --- Hex Info ---
        ui.heading("Hex Info");
        ui.add_space(4.0);

        if let Some(ref cell) = self.hovered_hex {
            ui.label(format!("Coord: ({}, {})", cell.q, cell.r));
            ui.label(format!("Elevation: {}", cell.elevation.label()));
            ui.label(format!("Climate: {}", cell.climate.label()));
            if !cell.decorators.is_empty() {
                let dec_labels: Vec<&str> =
                    cell.decorators.iter().map(|d| d.label()).collect();
                ui.label(format!("Decorators: {}", dec_labels.join(", ")));
            }
            if let Some(ref lbl) = cell.label {
                ui.label(format!("Label: {}", lbl));
            }
        } else {
            ui.label("Hover over a hex to see info");
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(4.0);

        // --- Brush Settings ---
        ui.heading("Brush Settings");
        ui.add_space(4.0);

        // Elevation combo
        let prev_elev = self.selected_elevation;
        egui::ComboBox::from_label("Elevation")
            .selected_text(self.selected_elevation.label())
            .show_ui(ui, |ui| {
                for &elev in Elevation::ALL.iter() {
                    ui.selectable_value(
                        &mut self.selected_elevation,
                        elev,
                        elev.label(),
                    );
                }
            });
        if self.selected_elevation != prev_elev {
            changed = true;
        }

        // Climate combo
        let prev_climate = self.selected_climate;
        egui::ComboBox::from_label("Climate")
            .selected_text(self.selected_climate.label())
            .show_ui(ui, |ui| {
                for &clim in Climate::ALL.iter() {
                    ui.selectable_value(
                        &mut self.selected_climate,
                        clim,
                        clim.label(),
                    );
                }
            });
        if self.selected_climate != prev_climate {
            changed = true;
        }

        // Radius slider
        let prev_radius = self.brush_radius;
        ui.add(egui::Slider::new(&mut self.brush_radius, 0..=5).text("Radius"));
        if self.brush_radius != prev_radius {
            changed = true;
        }

        changed
    }
}
