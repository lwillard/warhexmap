use crate::sprites::atlas::MultiPageAtlas;
use crate::sprites::clipboard::read_clipboard_image;
use crate::types::*;

pub struct SpriteEditorState {
    pub selected_terrain_idx: Option<usize>,
    pub selected_decorator_idx: Option<usize>,
    pub selected_slot: SpriteSlot,
    pub new_terrain_name: String,
    pub new_decorator_name: String,
    pub status_message: String,
}

const EDGE_LABELS: [&str; 6] = ["Edge N", "Edge NE", "Edge SE", "Edge S", "Edge SW", "Edge NW"];

impl SpriteEditorState {
    pub fn new() -> Self {
        Self {
            selected_terrain_idx: None,
            selected_decorator_idx: None,
            selected_slot: SpriteSlot::Base(0),
            new_terrain_name: String::new(),
            new_decorator_name: String::new(),
            status_message: String::new(),
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        atlas: &mut MultiPageAtlas,
        clipboard: &mut arboard::Clipboard,
    ) {
        // Left panel: terrain/decorator list
        egui::SidePanel::left("sprite_list_panel")
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Terrains");

                for (idx, terrain) in atlas.terrains.iter().enumerate() {
                    let selected = self.selected_terrain_idx == Some(idx);
                    if ui
                        .selectable_label(selected, &terrain.name)
                        .clicked()
                    {
                        self.selected_terrain_idx = Some(idx);
                        self.selected_decorator_idx = None;
                        self.selected_slot = SpriteSlot::Base(0);
                    }
                }

                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.new_terrain_name);
                    if ui.button("Add").clicked() && !self.new_terrain_name.is_empty() {
                        let priority = atlas.terrains.len() as u16 * 10;
                        let id = atlas.add_terrain(&self.new_terrain_name, priority);
                        // Generate a white hex base sprite as the initial variant
                        let white = [255u8, 255, 255, 255];
                        let sprite = crate::sprites::image_ops::generate_test_base_sprite(
                            crate::hex::geometry::SPRITE_SIZE, white,
                        );
                        atlas.insert_sprite(&sprite, SpriteLogicalKey::Base(id, 0));
                        self.new_terrain_name.clear();
                        self.status_message = "Terrain added".to_string();
                    }
                });

                if let Some(idx) = self.selected_terrain_idx {
                    if idx < atlas.terrains.len() {
                        if ui.button("Remove Terrain").clicked() {
                            atlas.terrains.remove(idx);
                            self.selected_terrain_idx = None;
                            self.status_message = "Terrain removed".to_string();
                        }
                    }
                }

                ui.separator();
                ui.heading("Decorators");

                for (idx, decorator) in atlas.decorators.iter().enumerate() {
                    let selected = self.selected_decorator_idx == Some(idx);
                    if ui
                        .selectable_label(selected, &decorator.name)
                        .clicked()
                    {
                        self.selected_decorator_idx = Some(idx);
                        self.selected_terrain_idx = None;
                    }
                }

                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.new_decorator_name);
                    if ui.button("Add").clicked() && !self.new_decorator_name.is_empty() {
                        atlas.add_decorator(&self.new_decorator_name, DecoratorCategory::Natural);
                        self.new_decorator_name.clear();
                        self.status_message = "Decorator added".to_string();
                    }
                });

                if let Some(idx) = self.selected_decorator_idx {
                    if idx < atlas.decorators.len() {
                        if ui.button("Remove Decorator").clicked() {
                            atlas.decorators.remove(idx);
                            self.selected_decorator_idx = None;
                            self.status_message = "Decorator removed".to_string();
                        }
                    }
                }
            });

        // Central area: sprite slot grid
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(terrain_idx) = self.selected_terrain_idx {
                if terrain_idx < atlas.terrains.len() {
                    let terrain_name = atlas.terrains[terrain_idx].name.clone();
                    let terrain_id = atlas.terrains[terrain_idx].id;
                    ui.heading(&terrain_name);
                    ui.separator();

                    // Row 0: Base variants (4 slots)
                    ui.label("Base");
                    ui.horizontal(|ui| {
                        for variant in 0..4u8 {
                            let slot = SpriteSlot::Base(variant);
                            let is_selected = self.selected_slot == slot;
                            let has_sprite = atlas.base_lut.contains_key(&(terrain_id, variant));

                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(60.0, 60.0),
                                egui::Sense::click(),
                            );

                            if response.clicked() {
                                self.selected_slot = slot;
                            }

                            // Draw background
                            let bg_color = if is_selected {
                                egui::Color32::from_rgb(80, 80, 120)
                            } else {
                                egui::Color32::from_rgb(50, 50, 50)
                            };
                            ui.painter().rect_filled(rect, 2.0, bg_color);

                            if has_sprite {
                                if let Some(loc) = atlas.base_lut.get(&(terrain_id, variant)) {
                                    if let Some((tex_id, uv)) = atlas.get_sprite_uv(loc) {
                                        ui.painter().image(
                                            tex_id,
                                            rect,
                                            uv,
                                            egui::Color32::WHITE,
                                        );
                                    }
                                }
                            } else {
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    format!("B{}", variant),
                                    egui::FontId::proportional(10.0),
                                    egui::Color32::GRAY,
                                );
                            }

                            // Selection outline
                            if is_selected {
                                ui.painter().rect_stroke(
                                    rect,
                                    2.0,
                                    egui::Stroke::new(2.0, egui::Color32::YELLOW),
                                );
                            }
                        }
                    });

                    // Rows 1-6: Edge overlays (6 edges x 4 variants)
                    for edge in 0..6u8 {
                        ui.label(EDGE_LABELS[edge as usize]);
                        ui.horizontal(|ui| {
                            for variant in 0..4u8 {
                                let slot = SpriteSlot::Edge(edge, variant);
                                let is_selected = self.selected_slot == slot;
                                let has_sprite =
                                    atlas.edge_lut.contains_key(&(terrain_id, edge, variant));

                                let (rect, response) = ui.allocate_exact_size(
                                    egui::vec2(60.0, 60.0),
                                    egui::Sense::click(),
                                );

                                if response.clicked() {
                                    self.selected_slot = slot;
                                }

                                let bg_color = if is_selected {
                                    egui::Color32::from_rgb(80, 80, 120)
                                } else {
                                    egui::Color32::from_rgb(50, 50, 50)
                                };
                                ui.painter().rect_filled(rect, 2.0, bg_color);

                                if has_sprite {
                                    if let Some(loc) =
                                        atlas.edge_lut.get(&(terrain_id, edge, variant))
                                    {
                                        if let Some((tex_id, uv)) = atlas.get_sprite_uv(loc) {
                                            ui.painter().image(
                                                tex_id,
                                                rect,
                                                uv,
                                                egui::Color32::WHITE,
                                            );
                                        }
                                    }
                                } else {
                                    ui.painter().text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        format!("E{}v{}", edge, variant),
                                        egui::FontId::proportional(10.0),
                                        egui::Color32::GRAY,
                                    );
                                }

                                if is_selected {
                                    ui.painter().rect_stroke(
                                        rect,
                                        2.0,
                                        egui::Stroke::new(2.0, egui::Color32::YELLOW),
                                    );
                                }
                            }
                        });
                    }

                    ui.separator();

                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.button("Paste from Clipboard").clicked() {
                            self.paste_clipboard_sprite(atlas, clipboard, terrain_id);
                        }
                        if ui.button("Clear Slot").clicked() {
                            self.clear_selected_slot(atlas, terrain_id);
                        }
                    });
                }
            } else if let Some(dec_idx) = self.selected_decorator_idx {
                if dec_idx < atlas.decorators.len() {
                    let dec_name = atlas.decorators[dec_idx].name.clone();
                    let dec_id = atlas.decorators[dec_idx].id;
                    ui.heading(&dec_name);
                    ui.separator();

                    ui.label("Decorator Variants");
                    ui.horizontal(|ui| {
                        for variant in 0..4u8 {
                            let has_sprite =
                                atlas.decorator_lut.contains_key(&(dec_id, variant));

                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(60.0, 60.0),
                                egui::Sense::click(),
                            );

                            if response.clicked() {
                                self.selected_slot = SpriteSlot::Base(variant);
                            }

                            let is_selected = self.selected_slot == SpriteSlot::Base(variant);
                            let bg_color = if is_selected {
                                egui::Color32::from_rgb(80, 80, 120)
                            } else {
                                egui::Color32::from_rgb(50, 50, 50)
                            };
                            ui.painter().rect_filled(rect, 2.0, bg_color);

                            if has_sprite {
                                if let Some(loc) = atlas.decorator_lut.get(&(dec_id, variant)) {
                                    if let Some((tex_id, uv)) = atlas.get_sprite_uv(loc) {
                                        ui.painter().image(
                                            tex_id,
                                            rect,
                                            uv,
                                            egui::Color32::WHITE,
                                        );
                                    }
                                }
                            } else {
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    format!("D{}", variant),
                                    egui::FontId::proportional(10.0),
                                    egui::Color32::GRAY,
                                );
                            }

                            if is_selected {
                                ui.painter().rect_stroke(
                                    rect,
                                    2.0,
                                    egui::Stroke::new(2.0, egui::Color32::YELLOW),
                                );
                            }
                        }
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Paste from Clipboard").clicked() {
                            self.paste_clipboard_decorator(atlas, clipboard, dec_id);
                        }
                        if ui.button("Clear Slot").clicked() {
                            // For decorators, selected_slot Base(v) maps to variant v
                            if let SpriteSlot::Base(v) = self.selected_slot {
                                atlas.decorator_lut.remove(&(dec_id, v));
                                self.status_message = "Decorator slot cleared".to_string();
                            }
                        }
                    });
                }
            } else {
                ui.label("Select a terrain or decorator from the list");
            }

            // Status message at bottom
            ui.separator();
            ui.label(&self.status_message);
        });
    }

    fn paste_clipboard_sprite(
        &mut self,
        atlas: &mut MultiPageAtlas,
        clipboard: &mut arboard::Clipboard,
        terrain_id: uuid::Uuid,
    ) {
        match read_clipboard_image(clipboard) {
            Ok(mut sprite) => {
                // Resize to sprite size if needed
                let sz = crate::hex::geometry::SPRITE_SIZE;
                if sprite.width != sz || sprite.height != sz {
                    sprite = crate::sprites::image_ops::resize_sprite(&sprite, sz, sz);
                }
                // Clip base sprites to the hex polygon; edge overlays are left unclipped
                // so they can bleed across hex boundaries
                let is_base = matches!(self.selected_slot, SpriteSlot::Base(_));
                if is_base {
                    crate::sprites::image_ops::clip_to_hex(&mut sprite, crate::hex::geometry::HEX_RADIUS);
                }
                let key = match self.selected_slot {
                    SpriteSlot::Base(v) => SpriteLogicalKey::Base(terrain_id, v),
                    SpriteSlot::Edge(e, v) => SpriteLogicalKey::Edge(terrain_id, e, v),
                };
                atlas.insert_sprite(&sprite, key);
                self.status_message = format!("Pasted sprite into {:?}", self.selected_slot);
            }
            Err(e) => {
                self.status_message = format!("Clipboard error: {}", e);
            }
        }
    }

    fn paste_clipboard_decorator(
        &mut self,
        atlas: &mut MultiPageAtlas,
        clipboard: &mut arboard::Clipboard,
        dec_id: uuid::Uuid,
    ) {
        match read_clipboard_image(clipboard) {
            Ok(sprite) => {
                if let SpriteSlot::Base(v) = self.selected_slot {
                    let key = SpriteLogicalKey::Decorator(dec_id, v);
                    atlas.insert_sprite(&sprite, key);
                    self.status_message = format!("Pasted decorator variant {}", v);
                }
            }
            Err(e) => {
                self.status_message = format!("Clipboard error: {}", e);
            }
        }
    }

    fn clear_selected_slot(&mut self, atlas: &mut MultiPageAtlas, terrain_id: uuid::Uuid) {
        match self.selected_slot {
            SpriteSlot::Base(v) => {
                atlas.base_lut.remove(&(terrain_id, v));
                self.status_message = format!("Cleared Base variant {}", v);
            }
            SpriteSlot::Edge(e, v) => {
                atlas.edge_lut.remove(&(terrain_id, e, v));
                self.status_message = format!("Cleared Edge {} variant {}", e, v);
            }
        }
    }
}
