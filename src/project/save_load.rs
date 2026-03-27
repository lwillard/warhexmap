use std::path::Path;
use serde::{Serialize, Deserialize};
use crate::hex::map::HexMap;
use crate::sprites::atlas::{MultiPageAtlas, AtlasPage};
use crate::types::*;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct ProjectFile {
    pub version: u32,
    pub atlas_metadata: AtlasMetadata,
    pub map: HexMap,
}

#[derive(Serialize, Deserialize)]
pub struct AtlasMetadata {
    pub page_count: u32,
    pub next_slots: Vec<u32>,
    pub terrains: Vec<TerrainDef>,
    pub decorators: Vec<DecoratorDef>,
    pub base_lut: Vec<BaseLutEntry>,
    pub edge_lut: Vec<EdgeLutEntry>,
    pub decorator_lut: Vec<DecoratorLutEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct BaseLutEntry {
    pub terrain_id: Uuid,
    pub variant: u8,
    pub sprite_ref: PackedSpriteRef,
}

#[derive(Serialize, Deserialize)]
pub struct EdgeLutEntry {
    pub terrain_id: Uuid,
    pub edge: u8,
    pub variant: u8,
    pub sprite_ref: PackedSpriteRef,
}

#[derive(Serialize, Deserialize)]
pub struct DecoratorLutEntry {
    pub decorator_id: Uuid,
    pub variant: u8,
    pub sprite_ref: PackedSpriteRef,
}

pub fn save_project(dir: &Path, map: &HexMap, atlas: &MultiPageAtlas) -> Result<(), String> {
    // Create directory if it doesn't exist
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create directory: {}", e))?;

    // Save each atlas page as a PNG
    for (i, page) in atlas.pages.iter().enumerate() {
        let path = dir.join(format!("atlas_page_{}.png", i));
        save_rgba_png(&path, &page.image.pixels, page.image.width, page.image.height)?;
    }

    // Build LUT vecs
    let base_lut: Vec<BaseLutEntry> = atlas.base_lut.iter().map(|((tid, v), sr)| {
        BaseLutEntry { terrain_id: *tid, variant: *v, sprite_ref: *sr }
    }).collect();

    let edge_lut: Vec<EdgeLutEntry> = atlas.edge_lut.iter().map(|((tid, e, v), sr)| {
        EdgeLutEntry { terrain_id: *tid, edge: *e, variant: *v, sprite_ref: *sr }
    }).collect();

    let decorator_lut: Vec<DecoratorLutEntry> = atlas.decorator_lut.iter().map(|((did, v), sr)| {
        DecoratorLutEntry { decorator_id: *did, variant: *v, sprite_ref: *sr }
    }).collect();

    let next_slots: Vec<u32> = atlas.pages.iter().map(|p| p.next_slot).collect();

    let project = ProjectFile {
        version: 1,
        atlas_metadata: AtlasMetadata {
            page_count: atlas.pages.len() as u32,
            next_slots,
            terrains: atlas.terrains.clone(),
            decorators: atlas.decorators.clone(),
            base_lut,
            edge_lut,
            decorator_lut,
        },
        map: map.clone(),
    };

    let json = serde_json::to_string_pretty(&project)
        .map_err(|e| format!("Failed to serialize project: {}", e))?;
    let json_path = dir.join("project.json");
    std::fs::write(&json_path, json)
        .map_err(|e| format!("Failed to write project.json: {}", e))?;

    Ok(())
}

pub fn load_project(dir: &Path) -> Result<(MultiPageAtlas, HexMap), String> {
    let json_path = dir.join("project.json");
    let json = std::fs::read_to_string(&json_path)
        .map_err(|e| format!("Failed to read project.json: {}", e))?;
    let project: ProjectFile = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse project.json: {}", e))?;

    let meta = project.atlas_metadata;

    // Load atlas page PNGs
    let mut pages = Vec::new();
    for i in 0..meta.page_count {
        let path = dir.join(format!("atlas_page_{}.png", i));
        let pixels = load_rgba_png(&path)?;
        let next_slot = meta.next_slots.get(i as usize).copied().unwrap_or(0);
        let width = 4096u32;
        let height = 4096u32;
        pages.push(AtlasPage {
            image: SpriteImage { width, height, pixels },
            texture_handle: None, // Will be rebuilt on first frame
            next_slot,
        });
    }

    // Reconstruct LUTs
    let mut base_lut = std::collections::HashMap::new();
    for entry in meta.base_lut {
        base_lut.insert((entry.terrain_id, entry.variant), entry.sprite_ref);
    }

    let mut edge_lut = std::collections::HashMap::new();
    for entry in meta.edge_lut {
        edge_lut.insert((entry.terrain_id, entry.edge, entry.variant), entry.sprite_ref);
    }

    let mut decorator_lut = std::collections::HashMap::new();
    for entry in meta.decorator_lut {
        decorator_lut.insert((entry.decorator_id, entry.variant), entry.sprite_ref);
    }

    let atlas = MultiPageAtlas {
        pages,
        terrains: meta.terrains,
        decorators: meta.decorators,
        base_lut,
        edge_lut,
        decorator_lut,
    };

    Ok((atlas, project.map))
}

fn save_rgba_png(path: &Path, pixels: &[u8], width: u32, height: u32) -> Result<(), String> {
    let img = image::RgbaImage::from_raw(width, height, pixels.to_vec())
        .ok_or_else(|| "Failed to create image buffer".to_string())?;
    img.save(path).map_err(|e| format!("Failed to save PNG {}: {}", path.display(), e))
}

fn load_rgba_png(path: &Path) -> Result<Vec<u8>, String> {
    let img = image::open(path)
        .map_err(|e| format!("Failed to load PNG {}: {}", path.display(), e))?;
    Ok(img.to_rgba8().into_raw())
}
