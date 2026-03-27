use crate::hex::map::HexMap;
use crate::sprites::atlas::MultiPageAtlas;
use crate::editor::sprite_editor::SpriteEditorState;
use crate::editor::map_editor::MapEditorState;
use crate::editor::undo::UndoStack;

#[derive(PartialEq)]
pub enum EditorMode {
    SpriteEditor,
    MapEditor,
}

pub struct HexEditorApp {
    pub mode: EditorMode,
    pub map: HexMap,
    pub atlas: MultiPageAtlas,
    pub undo: UndoStack,
    pub sprite_editor: SpriteEditorState,
    pub map_editor: MapEditorState,
    pub clipboard: arboard::Clipboard,
    pub project_path: Option<std::path::PathBuf>,
}

impl HexEditorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Create atlas with test sprites
        let mut atlas = MultiPageAtlas::new();
        atlas.generate_test_sprites();

        // Create default map using first terrain
        let default_terrain = atlas.terrains[0].id;
        let map = HexMap::new("Untitled", 20, 15, default_terrain);

        let clipboard = arboard::Clipboard::new().expect("Failed to init clipboard");

        Self {
            mode: EditorMode::MapEditor,
            map,
            atlas,
            undo: UndoStack::new(),
            sprite_editor: SpriteEditorState::new(),
            map_editor: MapEditorState::new(),
            clipboard,
            project_path: None,
        }
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Z) && !i.modifiers.shift {
                self.undo.undo(&mut self.map);
            }
            if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::Z) {
                self.undo.redo(&mut self.map);
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
                self.save();
            }
            // Tool shortcuts (only in map mode)
            if self.mode == EditorMode::MapEditor && !i.modifiers.ctrl {
                if i.key_pressed(egui::Key::B) {
                    self.map_editor.current_tool = crate::editor::tools::MapTool::PaintTerrain;
                }
                if i.key_pressed(egui::Key::E) {
                    self.map_editor.current_tool = crate::editor::tools::MapTool::Eraser;
                }
                if i.key_pressed(egui::Key::I) {
                    self.map_editor.current_tool = crate::editor::tools::MapTool::Eyedropper;
                }
                if i.key_pressed(egui::Key::G) {
                    self.map_editor.show_grid = !self.map_editor.show_grid;
                }
            }
        });
    }

    fn draw_menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New").clicked() {
                    self.new_project();
                    ui.close_menu();
                }
                if ui.button("Open...").clicked() {
                    self.open();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Save").clicked() {
                    self.save();
                    ui.close_menu();
                }
                if ui.button("Save As...").clicked() {
                    self.save_as();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui
                    .add_enabled(self.undo.can_undo(), egui::Button::new("Undo"))
                    .clicked()
                {
                    self.undo.undo(&mut self.map);
                    ui.close_menu();
                }
                if ui
                    .add_enabled(self.undo.can_redo(), egui::Button::new("Redo"))
                    .clicked()
                {
                    self.undo.redo(&mut self.map);
                    ui.close_menu();
                }
            });
        });
    }

    fn new_project(&mut self) {
        self.atlas = MultiPageAtlas::new();
        self.atlas.generate_test_sprites();
        let default_terrain = self.atlas.terrains[0].id;
        self.map = HexMap::new("Untitled", 20, 15, default_terrain);
        self.undo = UndoStack::new();
        self.project_path = None;
    }

    fn save(&mut self) {
        if let Some(path) = &self.project_path.clone() {
            if let Err(e) = crate::project::save_load::save_project(path, &self.map, &self.atlas) {
                log::error!("Save failed: {}", e);
            }
        } else {
            self.save_as();
        }
    }

    fn save_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Save Project")
            .pick_folder()
        {
            self.project_path = Some(path.clone());
            if let Err(e) =
                crate::project::save_load::save_project(&path, &self.map, &self.atlas)
            {
                log::error!("Save failed: {}", e);
            }
        }
    }

    fn open(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Open Project")
            .pick_folder()
        {
            match crate::project::save_load::load_project(&path) {
                Ok((atlas, map)) => {
                    self.atlas = atlas;
                    self.map = map;
                    self.undo = UndoStack::new();
                    self.project_path = Some(path);
                }
                Err(e) => log::error!("Load failed: {}", e),
            }
        }
    }
}

impl eframe::App for HexEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard_shortcuts(ctx);

        // Ensure atlas textures are uploaded
        self.atlas.ensure_textures(ctx);

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.draw_menu_bar(ui);
        });

        // Mode tabs
        egui::TopBottomPanel::top("mode_tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.mode, EditorMode::SpriteEditor, "Sprite Editor");
                ui.selectable_value(&mut self.mode, EditorMode::MapEditor, "Map Editor");
            });
        });

        match self.mode {
            EditorMode::SpriteEditor => {
                self.sprite_editor
                    .show(ctx, &mut self.atlas, &mut self.clipboard);
            }
            EditorMode::MapEditor => {
                self.map_editor
                    .show(ctx, &mut self.map, &mut self.atlas, &mut self.undo);
            }
        }
    }
}
