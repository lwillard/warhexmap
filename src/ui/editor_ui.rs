use crate::editor::brush_tool::PaintMode;
use crate::editor::tool_manager::ToolKind;
use crate::model::terrain_types::{Climate, Decorator, Elevation, PathType};
use crate::renderer::camera::Camera;
use crate::ui::minimap_widget::MinimapWidget;
use crate::ui::palette_panel::PaletteState;
use crate::ui::properties_panel::PropertiesState;
use crate::ui::toolbar::ToolbarState;

pub struct EditorUi {
    pub toolbar: ToolbarState,
    pub palette: PaletteState,
    pub properties: PropertiesState,
    pub status_text: String,
    pub show_grid: bool,

    // File dialog state
    pub pending_save: bool,
    pub pending_save_as: bool,
    pub pending_open: bool,
    pub pending_export: bool,
    pub pending_new: bool,

    // Edit actions
    pub pending_undo: bool,
    pub pending_redo: bool,
}

impl EditorUi {
    pub fn new() -> Self {
        Self {
            toolbar: ToolbarState::new(),
            palette: PaletteState::new(),
            properties: PropertiesState::new(),
            status_text: String::from("Ready"),
            show_grid: true,

            pending_save: false,
            pending_save_as: false,
            pending_open: false,
            pending_export: false,
            pending_new: false,

            pending_undo: false,
            pending_redo: false,
        }
    }

    /// Draw all UI panels. Returns the central panel rect where the map viewport renders.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        camera: &mut Camera,
        minimap_texture: Option<egui::TextureId>,
        world_bounds: (f32, f32, f32, f32),
    ) -> egui::Rect {
        // Reset per-frame action flags
        self.pending_save = false;
        self.pending_save_as = false;
        self.pending_open = false;
        self.pending_export = false;
        self.pending_new = false;
        self.pending_undo = false;
        self.pending_redo = false;

        // --- Top menu bar ---
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.pending_new = true;
                        ui.close_menu();
                    }
                    if ui.button("Open...").clicked() {
                        self.pending_open = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Save").clicked() {
                        self.pending_save = true;
                        ui.close_menu();
                    }
                    if ui.button("Save As...").clicked() {
                        self.pending_save_as = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Export PNG...").clicked() {
                        self.pending_export = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {
                        self.pending_undo = true;
                        ui.close_menu();
                    }
                    if ui.button("Redo").clicked() {
                        self.pending_redo = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui
                        .checkbox(&mut self.show_grid, "Show Grid")
                        .changed()
                    {
                        ui.close_menu();
                    }
                });
            });
        });

        // --- Bottom status bar ---
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_text);
            });
        });

        // --- Left panel: toolbar + palette + minimap ---
        egui::SidePanel::left("left_panel")
            .default_width(200.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.toolbar.show(ui);
                    ui.separator();
                    self.palette.show(ui);
                    ui.separator();

                    if let Some(tex_id) = minimap_texture {
                        MinimapWidget::show(ui, tex_id, camera, world_bounds);
                    }
                });
            });

        // --- Right panel: properties ---
        egui::SidePanel::right("right_panel")
            .default_width(220.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.properties.show(ui);
                });
            });

        // --- Central panel (map viewport) ---
        let central = egui::CentralPanel::default().show(ctx, |_ui| {});
        central.response.rect
    }
}
