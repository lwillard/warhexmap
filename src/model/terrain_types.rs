use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Elevation {
    VeryDeepWater = 0,
    DeepWater = 1,
    Water = 2,
    ShallowWater = 3,
    Plains = 4,
    Hills = 5,
    Mountains = 6,
}

impl Elevation {
    pub fn is_water(self) -> bool {
        (self as u8) <= 3
    }

    pub fn is_land(self) -> bool {
        !self.is_water()
    }

    pub fn texture_index(self, climate: Climate) -> u32 {
        if self.is_water() {
            self as u32
        } else {
            let band = match self {
                Self::Plains => 0,
                Self::Hills => 1,
                Self::Mountains => 2,
                _ => 0,
            };
            4 + band * 8 + climate.index() as u32
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::VeryDeepWater => "Very Deep Water",
            Self::DeepWater => "Deep Water",
            Self::Water => "Water",
            Self::ShallowWater => "Shallow Water",
            Self::Plains => "Plains",
            Self::Hills => "Hills",
            Self::Mountains => "Mountains",
        }
    }

    pub const ALL: [Elevation; 7] = [
        Self::VeryDeepWater,
        Self::DeepWater,
        Self::Water,
        Self::ShallowWater,
        Self::Plains,
        Self::Hills,
        Self::Mountains,
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Climate {
    BW,
    BS,
    Cs,
    Cw,
    Cf,
    Df,
    Am,
    Af,
}

impl Climate {
    pub fn index(self) -> usize {
        match self {
            Self::BW => 0,
            Self::BS => 1,
            Self::Cs => 2,
            Self::Cw => 3,
            Self::Cf => 4,
            Self::Df => 5,
            Self::Am => 6,
            Self::Af => 7,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::BW => "Arid Desert",
            Self::BS => "Arid Steppe",
            Self::Cs => "Mediterranean",
            Self::Cw => "Subtropical Highland",
            Self::Cf => "Oceanic/Humid",
            Self::Df => "Continental",
            Self::Am => "Tropical Monsoon",
            Self::Af => "Tropical Rainforest",
        }
    }

    pub const ALL: [Climate; 8] = [
        Self::BW,
        Self::BS,
        Self::Cs,
        Self::Cw,
        Self::Cf,
        Self::Df,
        Self::Am,
        Self::Af,
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Decorator {
    Grassland,
    Farms,
    Woods,
    DenseForest,
    Buildings,
    DenseBuildings,
}

impl Decorator {
    pub const ALL: [Decorator; 6] = [
        Self::Grassland,
        Self::Farms,
        Self::Woods,
        Self::DenseForest,
        Self::Buildings,
        Self::DenseBuildings,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Grassland => "Grassland",
            Self::Farms => "Farms",
            Self::Woods => "Woods",
            Self::DenseForest => "Dense Forest",
            Self::Buildings => "Buildings",
            Self::DenseBuildings => "Dense Buildings",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum PathType {
    DirtRoad,
    Road,
    MajorRoad,
    Rail,
    Stream,
    River,
    MajorRiver,
}

impl PathType {
    pub fn is_road(self) -> bool {
        matches!(
            self,
            Self::DirtRoad | Self::Road | Self::MajorRoad | Self::Rail
        )
    }

    pub fn is_water_feature(self) -> bool {
        matches!(self, Self::Stream | Self::River | Self::MajorRiver)
    }

    pub fn width(self) -> f32 {
        match self {
            Self::DirtRoad => 1.5,
            Self::Road => 2.0,
            Self::MajorRoad => 3.0,
            Self::Rail => 2.0,
            Self::Stream => 1.0,
            Self::River => 2.5,
            Self::MajorRiver => 5.0,
        }
    }

    pub fn color(self) -> [f32; 4] {
        match self {
            Self::DirtRoad => [0.549, 0.471, 0.353, 1.0],
            Self::Road => [0.510, 0.431, 0.314, 1.0],
            Self::MajorRoad => [0.471, 0.392, 0.275, 1.0],
            Self::Rail => [0.235, 0.235, 0.235, 1.0],
            Self::Stream => [0.314, 0.471, 0.667, 1.0],
            Self::River => [0.275, 0.431, 0.627, 1.0],
            Self::MajorRiver => [0.235, 0.392, 0.588, 1.0],
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::DirtRoad => "Dirt Road",
            Self::Road => "Road",
            Self::MajorRoad => "Major Road",
            Self::Rail => "Rail",
            Self::Stream => "Stream",
            Self::River => "River",
            Self::MajorRiver => "Major River",
        }
    }

    pub const ALL: [PathType; 7] = [
        Self::DirtRoad,
        Self::Road,
        Self::MajorRoad,
        Self::Rail,
        Self::Stream,
        Self::River,
        Self::MajorRiver,
    ];
}

/// Total number of terrain texture layers: 4 water + 3 land bands × 8 climates = 28.
pub const TERRAIN_TEXTURE_COUNT: u32 = 28;
