"""Color palette definitions for terrain rendering.

Köppen climate color definitions and water color gradients derived from
the reference wargame map style.
"""

from __future__ import annotations


# Water colors by elevation level
WATER_PALETTE = {
    0: {  # VERY_DEEP_WATER
        "base": (140, 165, 185),
        "light": (150, 175, 195),
        "dark": (125, 150, 175),
    },
    1: {  # DEEP_WATER
        "base": (145, 170, 190),
        "light": (155, 180, 200),
        "dark": (130, 155, 180),
    },
    2: {  # WATER
        "base": (155, 178, 198),
        "light": (165, 188, 205),
        "dark": (140, 165, 185),
    },
    3: {  # SHALLOW_WATER
        "base": (168, 190, 205),
        "light": (180, 200, 215),
        "dark": (155, 178, 195),
    },
}

# Land colors by climate type: {climate_name: {elevation_band: rgb}}
CLIMATE_PALETTE = {
    "BW": {  # Arid Desert
        "plains": (215, 200, 165),
        "hills": (200, 180, 145),
        "mountain": (180, 155, 120),
        "texture_noise_amplitude": 12,
    },
    "BS": {  # Arid Steppe
        "plains": (205, 195, 145),
        "hills": (190, 175, 130),
        "mountain": (170, 150, 115),
        "texture_noise_amplitude": 15,
    },
    "Cs": {  # Mediterranean
        "plains": (195, 190, 135),
        "hills": (180, 170, 120),
        "mountain": (160, 145, 105),
        "texture_noise_amplitude": 18,
    },
    "Cw": {  # Subtropical Highland
        "plains": (160, 180, 110),
        "hills": (145, 160, 100),
        "mountain": (130, 140, 90),
        "texture_noise_amplitude": 15,
    },
    "Cf": {  # Oceanic/Humid
        "plains": (140, 175, 100),
        "hills": (125, 155, 90),
        "mountain": (110, 135, 80),
        "texture_noise_amplitude": 20,
    },
    "Df": {  # Continental
        "plains": (155, 170, 115),
        "hills": (140, 155, 100),
        "mountain": (125, 135, 90),
        "texture_noise_amplitude": 15,
    },
    "Am": {  # Tropical Monsoon
        "plains": (100, 165, 80),
        "hills": (85, 145, 70),
        "mountain": (70, 125, 60),
        "texture_noise_amplitude": 22,
    },
    "Af": {  # Tropical Rainforest
        "plains": (75, 150, 65),
        "hills": (60, 130, 55),
        "mountain": (50, 110, 50),
        "texture_noise_amplitude": 25,
    },
}

# Decorator rendering colors
DECORATOR_COLORS = {
    "grassland": {
        "base": (155, 180, 110),
        "variation": 15,
    },
    "farms": {
        "colors": [
            (195, 200, 120),
            (185, 195, 110),
            (200, 195, 130),
            (190, 185, 115),
            (180, 190, 105),
        ],
    },
    "woods": {
        "tree_color": (70, 110, 55),
        "shadow_color": (50, 85, 40),
        "density": 0.3,
        "cluster_size": (3, 6),
    },
    "dense_forest": {
        "canopy_color": (55, 100, 45),
        "shadow_color": (35, 75, 30),
        "density": 0.8,
        "cluster_size": (4, 8),
    },
    "buildings": {
        "wall_color": (165, 155, 140),
        "roof_color": (145, 110, 85),
        "density": 0.15,
    },
    "dense_buildings": {
        "wall_color": (155, 145, 130),
        "roof_color": (135, 100, 75),
        "street_color": (140, 135, 120),
        "density": 0.6,
    },
}

# Hex grid overlay style
HEX_GRID_STYLE = {
    "color": (80, 80, 80, 80),
    "width": 1.0,
    "show_at_zoom": (1, 4),
}

# Road rendering styles
ROAD_STYLES = {
    "dirt_road": {
        "color": (140, 120, 90),
        "width": 1.5,
        "dash": (6, 4),
        "outline": None,
    },
    "road": {
        "color": (130, 110, 80),
        "width": 2.0,
        "dash": None,
        "outline": (100, 85, 60),
    },
    "major_road": {
        "color": (120, 100, 70),
        "width": 3.0,
        "dash": None,
        "outline": (90, 75, 50),
    },
    "rail": {
        "color": (60, 60, 60),
        "width": 2.0,
        "dash": None,
        "cross_ties": True,
    },
}

# River rendering styles
RIVER_STYLES = {
    "stream": {
        "color": (80, 120, 170),
        "width": 1.0,
        "width_variation": 0.2,
        "outline": None,
    },
    "river": {
        "color": (70, 110, 160),
        "width": 2.5,
        "width_variation": 0.3,
        "outline": (50, 85, 130),
    },
    "major_river": {
        "color": (60, 100, 150),
        "width": 5.0,
        "width_variation": 0.4,
        "outline": (40, 75, 120),
    },
}

# Shore/beach transition color
SHORE_COLOR = (200, 190, 160)
