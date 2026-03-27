use crate::editor::tools::{self, MapTool};
use crate::editor::undo::{UndoAction, UndoStack};
use crate::hex::geometry::*;
use crate::hex::map::HexMap;
use crate::sprites::atlas::MultiPageAtlas;

pub struct MapEditorState {
    pub current_tool: MapTool,
    pub paint_terrain_idx: Option<usize>,
    pub paint_decorator_idx: Option<usize>,
    pub camera_offset: egui::Vec2,
    pub camera_zoom: f32,
    pub show_grid: bool,
    pub show_coords: bool,
    pub brush_size: u8,
    pub status_text: String,
    pub hovered_hex: Option<(i32, i32)>,
}

impl MapEditorState {
    pub fn new() -> Self {
        Self {
            current_tool: MapTool::PaintTerrain,
            paint_terrain_idx: Some(0),
            paint_decorator_idx: None,
            camera_offset: egui::Vec2::ZERO,
            camera_zoom: 1.0,
            show_grid: true,
            show_coords: false,
            brush_size: 1,
            status_text: String::from("Ready"),
            hovered_hex: None,
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        map: &mut HexMap,
        atlas: &mut MultiPageAtlas,
        undo: &mut UndoStack,
    ) {
        // Left panel: tools + terrain palette
        egui::SidePanel::left("map_tools_panel")
            .default_width(180.0)
            .show(ctx, |ui| {
                self.draw_tool_palette(ui);
                ui.separator();
                self.draw_terrain_palette(ui, atlas);
                ui.separator();
                self.draw_decorator_palette(ui, atlas);
                ui.separator();
                // Brush size slider (1-5)
                ui.add(egui::Slider::new(&mut self.brush_size, 1..=5u8).text("Brush"));
                // Grid/coords toggles
                ui.checkbox(&mut self.show_grid, "Show Grid");
                ui.checkbox(&mut self.show_coords, "Show Coords");
            });

        // Bottom status bar
        egui::TopBottomPanel::bottom("map_status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_text);
            });
        });

        // Central panel: map viewport
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_map_viewport(ui, map, atlas, undo);
        });
    }

    fn draw_tool_palette(&mut self, ui: &mut egui::Ui) {
        ui.heading("Tools");

        let tools = [
            (MapTool::PaintTerrain, "Paint Terrain", "P"),
            (MapTool::PaintDecorator, "Paint Decorator", "D"),
            (MapTool::Eraser, "Eraser", "E"),
            (MapTool::Eyedropper, "Eyedropper", "I"),
            (MapTool::Label, "Label", "L"),
        ];

        for (tool, name, shortcut) in &tools {
            let label = format!("{} [{}]", name, shortcut);
            if ui
                .selectable_label(self.current_tool == *tool, label)
                .clicked()
            {
                self.current_tool = *tool;
            }
        }
    }

    fn draw_terrain_palette(&mut self, ui: &mut egui::Ui, atlas: &MultiPageAtlas) {
        ui.heading("Terrain");

        for (idx, terrain) in atlas.terrains.iter().enumerate() {
            let selected = self.paint_terrain_idx == Some(idx);
            if ui.selectable_label(selected, &terrain.name).clicked() {
                self.paint_terrain_idx = Some(idx);
                self.current_tool = MapTool::PaintTerrain;
            }
        }
    }

    fn draw_decorator_palette(&mut self, ui: &mut egui::Ui, atlas: &MultiPageAtlas) {
        ui.heading("Decorators");

        for (idx, decorator) in atlas.decorators.iter().enumerate() {
            let selected = self.paint_decorator_idx == Some(idx);
            if ui.selectable_label(selected, &decorator.name).clicked() {
                self.paint_decorator_idx = Some(idx);
                self.current_tool = MapTool::PaintDecorator;
            }
        }
    }

    fn draw_map_viewport(
        &mut self,
        ui: &mut egui::Ui,
        map: &mut HexMap,
        atlas: &mut MultiPageAtlas,
        undo: &mut UndoStack,
    ) {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let viewport_rect = response.rect;

        // Handle pan (middle mouse drag or right-click drag)
        if response.dragged_by(egui::PointerButton::Middle) {
            self.camera_offset += response.drag_delta();
        }
        if response.dragged_by(egui::PointerButton::Secondary) {
            self.camera_offset += response.drag_delta();
        }

        // Handle zoom (scroll wheel)
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 && response.hovered() {
            let old_zoom = self.camera_zoom;
            self.camera_zoom = (self.camera_zoom * (1.0 + scroll * 0.001)).clamp(0.1, 5.0);
            // Zoom toward cursor
            if let Some(mouse) = response.hover_pos() {
                let mx = mouse.x - viewport_rect.left();
                let my = mouse.y - viewport_rect.top();
                self.camera_offset.x =
                    mx - (mx - self.camera_offset.x) * (self.camera_zoom / old_zoom);
                self.camera_offset.y =
                    my - (my - self.camera_offset.y) * (self.camera_zoom / old_zoom);
            }
        }

        // Determine visible hex range
        let (min_q, min_r, max_q, max_r) = self.visible_hex_range(viewport_rect, map);

        // Render visible hexes
        for r in min_r..=max_r {
            for q in min_q..=max_q {
                self.render_hex(&painter, viewport_rect, q, r, map, atlas);
            }
        }

        // Handle hover
        if let Some(pos) = response.hover_pos() {
            let world_x =
                (pos.x - viewport_rect.left() - self.camera_offset.x) / self.camera_zoom;
            let world_y =
                (pos.y - viewport_rect.top() - self.camera_offset.y) / self.camera_zoom;
            let (q, r) = pixel_to_hex(world_x, world_y);
            self.hovered_hex = Some((q, r));

            let terrain_name = map
                .try_get(q, r)
                .and_then(|cell| {
                    atlas
                        .terrains
                        .iter()
                        .find(|t| t.id == cell.terrain_id)
                })
                .map(|t| t.name.as_str())
                .unwrap_or("(empty)");
            self.status_text = format!("Hex: ({}, {})  Terrain: {}", q, r, terrain_name);
        }

        // Handle click/drag to paint
        if response.clicked()
            || (response.dragged_by(egui::PointerButton::Primary)
                && !ui.input(|i| i.modifiers.ctrl))
        {
            if let Some(pos) = response.interact_pointer_pos() {
                let world_x =
                    (pos.x - viewport_rect.left() - self.camera_offset.x) / self.camera_zoom;
                let world_y =
                    (pos.y - viewport_rect.top() - self.camera_offset.y) / self.camera_zoom;
                let (q, r) = pixel_to_hex(world_x, world_y);
                self.handle_tool_click(q, r, map, atlas, undo);
            }
        }
    }

    fn render_hex(
        &self,
        painter: &egui::Painter,
        viewport: egui::Rect,
        q: i32,
        r: i32,
        map: &HexMap,
        atlas: &MultiPageAtlas,
    ) {
        let (wx, wy) = hex_to_pixel(q, r);
        let sx = viewport.left() + self.camera_offset.x + wx * self.camera_zoom;
        let sy = viewport.top() + self.camera_offset.y + wy * self.camera_zoom;

        let half = SPRITE_SIZE as f32 * self.camera_zoom * 0.5;
        let rect = egui::Rect::from_min_size(
            egui::pos2(sx - half, sy - half),
            egui::vec2(half * 2.0, half * 2.0),
        );

        if let Some(cell) = map.try_get(q, r) {
            // 1. Draw base terrain sprite
            if let Some(loc) = atlas.base_lut.get(&(cell.terrain_id, cell.variant_index)) {
                if let Some((tex_id, uv)) = atlas.get_sprite_uv(loc) {
                    painter.image(tex_id, rect, uv, egui::Color32::WHITE);
                }
            }

            // 2. Draw edge overlays from neighbors
            for edge in 0..6usize {
                let (nq, nr) = (q + AXIAL_NEIGHBORS[edge].0, r + AXIAL_NEIGHBORS[edge].1);
                if let Some(neighbor) = map.try_get(nq, nr) {
                    if neighbor.terrain_id != cell.terrain_id {
                        // Check priority
                        let cell_priority = atlas
                            .terrains
                            .iter()
                            .find(|t| t.id == cell.terrain_id)
                            .map(|t| t.priority)
                            .unwrap_or(0);
                        let neighbor_priority = atlas
                            .terrains
                            .iter()
                            .find(|t| t.id == neighbor.terrain_id)
                            .map(|t| t.priority)
                            .unwrap_or(0);
                        if neighbor_priority > cell_priority {
                            let opp = opposite_edge(edge);
                            if let Some(loc) = atlas.edge_lut.get(&(
                                neighbor.terrain_id,
                                opp as u8,
                                cell.edge_variants[edge],
                            )) {
                                if let Some((tex_id, uv)) = atlas.get_sprite_uv(loc) {
                                    painter.image(tex_id, rect, uv, egui::Color32::WHITE);
                                }
                            }
                        }
                    }
                }
            }

            // 3. Draw decorators
            for dec in &cell.decorators {
                if let Some(loc) = atlas
                    .decorator_lut
                    .get(&(dec.decorator_id, dec.variant_index))
                {
                    if let Some((tex_id, uv)) = atlas.get_sprite_uv(loc) {
                        let dec_rect = rect.translate(egui::vec2(
                            dec.offset_x * self.camera_zoom,
                            dec.offset_y * self.camera_zoom,
                        ));
                        painter.image(tex_id, dec_rect, uv, egui::Color32::WHITE);
                    }
                }
            }

            // 4. Grid overlay
            if self.show_grid {
                let hex_pts = hex_polygon_points(sx, sy, HEX_RADIUS * self.camera_zoom);
                painter.add(egui::Shape::closed_line(
                    hex_pts,
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 80)),
                ));
            }

            // 5. Coordinate labels
            if self.show_coords {
                painter.text(
                    egui::pos2(sx, sy),
                    egui::Align2::CENTER_CENTER,
                    format!("{},{}", q, r),
                    egui::FontId::proportional(10.0 * self.camera_zoom),
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 120),
                );
            }
        }
    }

    fn visible_hex_range(&self, viewport: egui::Rect, map: &HexMap) -> (i32, i32, i32, i32) {
        let inv_zoom = 1.0 / self.camera_zoom;
        let margin = SPRITE_SIZE as f32;
        let left = (-self.camera_offset.x - margin) * inv_zoom;
        let right = (-self.camera_offset.x + viewport.width() + margin) * inv_zoom;
        let top = (-self.camera_offset.y - margin) * inv_zoom;
        let bottom = (-self.camera_offset.y + viewport.height() + margin) * inv_zoom;
        let min_q = (left / HEX_HORIZ).floor() as i32 - 1;
        let max_q = (right / HEX_HORIZ).ceil() as i32 + 1;
        let min_r = (top / HEX_VERT).floor() as i32 - 1;
        let max_r = (bottom / HEX_VERT).ceil() as i32 + 1;
        (
            min_q.max(0),
            min_r.max(0),
            max_q.min(map.width as i32 - 1),
            max_r.min(map.height as i32 - 1),
        )
    }

    fn handle_tool_click(
        &mut self,
        q: i32,
        r: i32,
        map: &mut HexMap,
        atlas: &MultiPageAtlas,
        undo: &mut UndoStack,
    ) {
        match self.current_tool {
            MapTool::PaintTerrain => {
                if let Some(idx) = self.paint_terrain_idx {
                    if let Some(terrain) = atlas.terrains.get(idx) {
                        let terrain_id = terrain.id;
                        let coords = tools::brush_coords(q, r, self.brush_size);
                        for (bq, br) in coords {
                            if let Some(old_cell) = map.try_get(bq, br).cloned() {
                                undo.push(UndoAction::SetCell {
                                    q: bq,
                                    r: br,
                                    old: old_cell,
                                });
                                let base_count = atlas.count_base_variants(terrain_id);
                                let mut edge_counts = [0u8; 6];
                                for e in 0..6 {
                                    edge_counts[e] =
                                        atlas.count_edge_variants(terrain_id, e as u8);
                                }
                                map.set_terrain(bq, br, terrain_id, base_count, &edge_counts);
                            }
                        }
                    }
                }
            }
            MapTool::PaintDecorator => {
                if let Some(idx) = self.paint_decorator_idx {
                    if let Some(decorator) = atlas.decorators.get(idx) {
                        let dec_id = decorator.id;
                        let variant_count = atlas.count_decorator_variants(dec_id);
                        if variant_count > 0 {
                            if let Some(cell) = map.cells.get_mut(&(q, r)) {
                                let mut rng = rand::thread_rng();
                                let variant =
                                    rand::Rng::gen_range(&mut rng, 0..variant_count);
                                cell.decorators
                                    .push(crate::hex::map::DecoratorPlacement {
                                        decorator_id: dec_id,
                                        variant_index: variant,
                                        offset_x: 0.0,
                                        offset_y: 0.0,
                                    });
                            }
                        }
                    }
                }
            }
            MapTool::Eraser => {
                if let Some(cell) = map.cells.get_mut(&(q, r)) {
                    if !cell.decorators.is_empty() {
                        cell.decorators.pop();
                    } else {
                        cell.terrain_id = map.default_terrain;
                        cell.variant_index = 0;
                    }
                }
            }
            MapTool::Eyedropper => {
                if let Some(cell) = map.try_get(q, r) {
                    let terrain_id = cell.terrain_id;
                    if let Some(idx) = atlas.terrains.iter().position(|t| t.id == terrain_id) {
                        self.paint_terrain_idx = Some(idx);
                        self.current_tool = MapTool::PaintTerrain;
                    }
                }
            }
            MapTool::Label => {
                // Label tool: no-op for now, would need text input dialog
            }
        }
    }
}
