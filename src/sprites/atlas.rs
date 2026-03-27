use std::collections::HashMap;
use uuid::Uuid;

use crate::hex::geometry::SPRITE_SIZE;
use crate::sprites::image_ops;
use crate::types::*;

const ATLAS_PAGE_SIZE: u32 = 4096;
const CELLS_PER_ROW: u32 = ATLAS_PAGE_SIZE / SPRITE_SIZE; // 24

pub struct AtlasPage {
    pub image: SpriteImage,
    pub texture_handle: Option<egui::TextureHandle>,
    pub next_slot: u32,
}

pub struct MultiPageAtlas {
    pub pages: Vec<AtlasPage>,
    pub terrains: Vec<TerrainDef>,
    pub decorators: Vec<DecoratorDef>,
    pub base_lut: HashMap<(Uuid, u8), PackedSpriteRef>,
    pub edge_lut: HashMap<(Uuid, u8, u8), PackedSpriteRef>,
    pub decorator_lut: HashMap<(Uuid, u8), PackedSpriteRef>,
}

impl MultiPageAtlas {
    /// Creates an empty atlas with no pages.
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            terrains: Vec::new(),
            decorators: Vec::new(),
            base_lut: HashMap::new(),
            edge_lut: HashMap::new(),
            decorator_lut: HashMap::new(),
        }
    }

    /// Allocate the next free cell, creating a new page if needed.
    pub fn alloc_cell(&mut self) -> PackedSpriteRef {
        let cells_per_page = CELLS_PER_ROW * CELLS_PER_ROW;

        // Find a page with a free slot, or create one
        let page_idx = self
            .pages
            .iter()
            .position(|p| p.next_slot < cells_per_page);

        let page_idx = match page_idx {
            Some(idx) => idx,
            None => {
                // Create a new page
                let page = AtlasPage {
                    image: SpriteImage {
                        width: ATLAS_PAGE_SIZE,
                        height: ATLAS_PAGE_SIZE,
                        pixels: vec![0u8; (ATLAS_PAGE_SIZE * ATLAS_PAGE_SIZE * 4) as usize],
                    },
                    texture_handle: None,
                    next_slot: 0,
                };
                self.pages.push(page);
                self.pages.len() - 1
            }
        };

        let slot = self.pages[page_idx].next_slot;
        self.pages[page_idx].next_slot += 1;

        let cell_x = slot % CELLS_PER_ROW;
        let cell_y = slot / CELLS_PER_ROW;

        PackedSpriteRef {
            page: page_idx as u16,
            cell_x: cell_x as u16,
            cell_y: cell_y as u16,
        }
    }

    /// Insert a sprite into the atlas: blit pixels, register in the appropriate LUT,
    /// and mark the page texture as dirty.
    pub fn insert_sprite(
        &mut self,
        sprite: &SpriteImage,
        key: SpriteLogicalKey,
    ) -> PackedSpriteRef {
        let loc = self.alloc_cell();

        // Resize sprite to SPRITE_SIZE if needed
        let fitted = if sprite.width != SPRITE_SIZE || sprite.height != SPRITE_SIZE {
            image_ops::resize_sprite(sprite, SPRITE_SIZE, SPRITE_SIZE)
        } else {
            sprite.clone()
        };

        // Blit into the atlas page
        let dx = loc.cell_x as u32 * SPRITE_SIZE;
        let dy = loc.cell_y as u32 * SPRITE_SIZE;
        let page = &mut self.pages[loc.page as usize];
        image_ops::blit(&mut page.image.pixels, ATLAS_PAGE_SIZE, &fitted, dx, dy);

        // Mark texture dirty
        page.texture_handle = None;

        // Register in LUT
        match key {
            SpriteLogicalKey::Base(terrain_id, variant) => {
                self.base_lut.insert((terrain_id, variant), loc);
            }
            SpriteLogicalKey::Edge(terrain_id, edge, variant) => {
                self.edge_lut.insert((terrain_id, edge, variant), loc);
            }
            SpriteLogicalKey::Decorator(dec_id, variant) => {
                self.decorator_lut.insert((dec_id, variant), loc);
            }
        }

        loc
    }

    /// Get texture ID and UV rect for a packed sprite.
    /// Returns None if the texture has not been uploaded yet.
    pub fn get_sprite_uv(&self, loc: &PackedSpriteRef) -> Option<(egui::TextureId, egui::Rect)> {
        let page = self.pages.get(loc.page as usize)?;
        let handle = page.texture_handle.as_ref()?;

        let px = loc.cell_x as f32 * SPRITE_SIZE as f32;
        let py = loc.cell_y as f32 * SPRITE_SIZE as f32;

        let uv = egui::Rect::from_min_size(
            egui::pos2(
                px / ATLAS_PAGE_SIZE as f32,
                py / ATLAS_PAGE_SIZE as f32,
            ),
            egui::vec2(
                SPRITE_SIZE as f32 / ATLAS_PAGE_SIZE as f32,
                SPRITE_SIZE as f32 / ATLAS_PAGE_SIZE as f32,
            ),
        );

        Some((handle.id(), uv))
    }

    /// Upload any dirty pages (texture_handle == None) to the GPU.
    pub fn ensure_textures(&mut self, ctx: &egui::Context) {
        for page in &mut self.pages {
            if page.texture_handle.is_none() {
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [ATLAS_PAGE_SIZE as usize, ATLAS_PAGE_SIZE as usize],
                    &page.image.pixels,
                );
                let handle = ctx.load_texture(
                    "atlas_page",
                    color_image,
                    egui::TextureOptions {
                        magnification: egui::TextureFilter::Nearest,
                        minification: egui::TextureFilter::Linear,
                        ..Default::default()
                    },
                );
                page.texture_handle = Some(handle);
            }
        }
    }

    /// Count how many base variants exist for a terrain.
    pub fn count_base_variants(&self, terrain_id: Uuid) -> u8 {
        let mut count = 0u8;
        while self.base_lut.contains_key(&(terrain_id, count)) {
            count += 1;
        }
        count
    }

    /// Count how many edge variants exist for a terrain on a given edge.
    pub fn count_edge_variants(&self, terrain_id: Uuid, edge: u8) -> u8 {
        let mut count = 0u8;
        while self.edge_lut.contains_key(&(terrain_id, edge, count)) {
            count += 1;
        }
        count
    }

    /// Count how many decorator variants exist.
    pub fn count_decorator_variants(&self, dec_id: Uuid) -> u8 {
        let mut count = 0u8;
        while self.decorator_lut.contains_key(&(dec_id, count)) {
            count += 1;
        }
        count
    }

    /// Create a new terrain definition and return its ID.
    pub fn add_terrain(&mut self, name: &str, priority: u16) -> Uuid {
        let id = Uuid::new_v4();
        self.terrains.push(TerrainDef {
            id,
            name: name.to_string(),
            priority,
        });
        id
    }

    /// Create a new decorator definition and return its ID.
    pub fn add_decorator(&mut self, name: &str, category: DecoratorCategory) -> Uuid {
        let id = Uuid::new_v4();
        self.decorators.push(DecoratorDef {
            id,
            name: name.to_string(),
            category,
        });
        id
    }

    /// Generate 4 test terrains with colored hex base sprites and edge overlays.
    pub fn generate_test_sprites(&mut self) {
        let terrains = [
            ("Forest", 100u16, [34u8, 139, 34, 255]),
            ("Plains", 50, [194, 178, 128, 255]),
            ("Water", 10, [65, 105, 225, 255]),
            ("Mountain", 200, [139, 137, 137, 255]),
        ];

        for (name, priority, color) in &terrains {
            let id = self.add_terrain(name, *priority);

            // Generate 2 base variants with slight color variation
            for variant in 0..2u8 {
                let mut c = *color;
                // Slightly vary the color for the second variant
                if variant == 1 {
                    c[0] = c[0].saturating_add(15);
                    c[1] = c[1].saturating_add(10);
                }
                let sprite = image_ops::generate_test_base_sprite(SPRITE_SIZE, c);
                self.insert_sprite(&sprite, SpriteLogicalKey::Base(id, variant));
            }

            // Generate edge overlays for each of the 6 edges
            let edge_color = [
                color[0].saturating_sub(30),
                color[1].saturating_sub(30),
                color[2].saturating_sub(30),
                180,
            ];
            for edge in 0..6u8 {
                let sprite =
                    image_ops::generate_test_edge_overlay(SPRITE_SIZE, edge_color, edge as usize);
                self.insert_sprite(&sprite, SpriteLogicalKey::Edge(id, edge, 0));
            }
        }
    }
}
