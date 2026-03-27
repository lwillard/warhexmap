use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single RGBA image stored in memory
#[derive(Clone)]
pub struct SpriteImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // RGBA, row-major, 4 bytes per pixel
}

/// Identifies a slot in a terrain's sprite set
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpriteSlot {
    Base(u8),          // variant 0..3
    Edge(u8, u8),      // (edge_index 0..5, variant 0..3)
}

/// A terrain type definition
#[derive(Serialize, Deserialize, Clone)]
pub struct TerrainDef {
    pub id: Uuid,
    pub name: String,
    pub priority: u16,
}

/// A decorator type definition
#[derive(Serialize, Deserialize, Clone)]
pub struct DecoratorDef {
    pub id: Uuid,
    pub name: String,
    pub category: DecoratorCategory,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum DecoratorCategory {
    Natural,
    Settlement,
    Road,
    River,
    Custom(String),
}

/// Reference to a sprite's packed location in the atlas
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct PackedSpriteRef {
    pub page: u16,
    pub cell_x: u16,
    pub cell_y: u16,
}

/// Logical key for looking up sprites
#[derive(Clone)]
pub enum SpriteLogicalKey {
    Base(Uuid, u8),
    Edge(Uuid, u8, u8),
    Decorator(Uuid, u8),
}
