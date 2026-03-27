# Hex Map & Sprite Editor - Instructions

## Getting Started

### Prerequisites

- Rust toolchain (install from https://rustup.rs)
- Windows, macOS, or Linux

### Building & Running

```bash
cd warhexmap
cargo run
```

The editor opens with a 20x15 hex map pre-filled with 4 test terrains (Forest, Plains, Water, Mountain).

---

## Editor Modes

The application has two tabs at the top:

- **Sprite Editor** - manage terrain/decorator sprite assets
- **Map Editor** - paint and edit hex maps

---

## Sprite Editor

### Overview

The Sprite Editor is where you define terrain types, decorator types, and import sprite artwork. Each terrain has up to 28 sprite slots (4 base variants + 6 edges x 4 variants). Each decorator has up to 4 variant slots.

### Layout

| Area | Contents |
|------|----------|
| Left panel | Terrain list, Decorator list, Add/Remove buttons |
| Central panel | Sprite slot grid, Paste/Clear buttons, Status |

### Adding a Terrain Type

1. Type a name in the text field under the terrain list
2. Click **Add**
3. The terrain is created with a white hex base sprite (variant 0)

### Adding a Decorator Type

1. Type a name in the text field under the decorator list
2. Click **Add**

### Importing Sprites from Photoshop

Sprites are 170x170 pixel RGBA images. The hex polygon is inscribed within this square canvas (radius = 85px, hex height ~147px, with ~11.5px transparent padding top/bottom).

1. In Photoshop (or any image editor), create a 170x170 canvas
2. Paint your terrain or edge overlay artwork
3. Copy to clipboard (Ctrl+C)
4. In the Sprite Editor, select a terrain from the left panel
5. Click on the desired slot in the grid:
   - **Row 0 (Base)**: The full hex fill sprite (4 variant columns)
   - **Rows 1-6 (Edge N/NE/SE/S/SW/NW)**: Edge overlay sprites (4 variant columns each)
6. Click **Paste from Clipboard**

**Important**: Base sprites are automatically clipped to the hex polygon shape. Edge overlays are NOT clipped, so they can bleed across hex boundaries for seamless transitions.

If the pasted image is not 170x170, it is automatically resized with bilinear interpolation.

### Sprite Slot Grid

```
Row 0: Base    [v0] [v1] [v2] [v3]    <- hex fill sprites
Row 1: Edge N  [v0] [v1] [v2] [v3]    <- bleed overlays
Row 2: Edge NE [v0] [v1] [v2] [v3]
Row 3: Edge SE [v0] [v1] [v2] [v3]
Row 4: Edge S  [v0] [v1] [v2] [v3]
Row 5: Edge SW [v0] [v1] [v2] [v3]
Row 6: Edge NW [v0] [v1] [v2] [v3]
```

### How Edge Overlays Work

When two different terrains share a hex edge, the higher-priority terrain's edge overlay is drawn on top of the lower-priority terrain's base sprite. This creates a seamless transition where, for example, forest texture bleeds into a plains hex along their shared boundary.

Each edge overlay should show the terrain "bleeding in" from one direction, with alpha fading toward the hex center. Only the strip along that edge should have content; the rest should be transparent.

### Terrain Priority

Higher-priority terrains overlay on top of lower ones:

| Terrain | Typical Priority |
|---------|-----------------|
| Water | 10 (base layer) |
| Plains | 50 |
| Forest | 100 |
| Mountain | 200 (overlays everything) |

---

## Map Editor

### Overview

The Map Editor is where you paint terrain and decorators onto the hex grid.

### Layout

| Area | Contents |
|------|----------|
| Left panel | Tool palette, Terrain palette, Decorator palette, Brush size, Grid/Coords toggles |
| Central area | Zoomable/pannable hex map viewport |
| Bottom bar | Hover info (hex coords, terrain name) |

### Navigation

| Action | Control |
|--------|---------|
| Pan | Middle-mouse drag or Right-click drag |
| Zoom | Scroll wheel (0.1x to 5.0x) |
| Zoom toward cursor | Scroll zooms centered on mouse position |

### Tools

| Tool | Shortcut | Description |
|------|----------|-------------|
| Paint Terrain | **B** | Paint the selected terrain onto hexes |
| Paint Decorator | **D** | Place the selected decorator on hexes |
| Eraser | **E** | Remove decorators first, then reset terrain to default |
| Eyedropper | **I** | Sample terrain from a hex and switch to Paint mode |
| Label | **L** | (Placeholder for future text label support) |

### Painting Terrain

1. Select a terrain from the **Terrain** palette in the left panel
2. Adjust **Brush** size (1-5):
   - 1 = single hex
   - 2 = center + 1 ring (7 hexes)
   - 3 = center + 2 rings (19 hexes)
   - 4 = center + 3 rings (37 hexes)
   - 5 = center + 4 rings (61 hexes)
3. Click or drag on the map to paint
4. Clicking the same terrain on a hex cycles through available variants

### Placing Decorators

1. Select a decorator from the **Decorators** palette
2. Click on hexes to place decorator instances
3. Each click adds one decorator with a random variant

### Erasing

1. Select the Eraser tool (or press **E**)
2. Click a hex:
   - If decorators exist, removes the topmost one
   - If no decorators, resets terrain to the map default

### Eyedropper

1. Select the Eyedropper tool (or press **I**)
2. Click a hex to sample its terrain
3. Automatically switches to Paint Terrain mode with the sampled terrain selected

### Display Options

- **Show Grid** checkbox: toggles hex grid outline overlay
- **Show Coords** checkbox: toggles axial coordinate labels on each hex

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| **Ctrl+Z** | Undo |
| **Ctrl+Shift+Z** | Redo |
| **Ctrl+S** | Save project |
| **B** | Paint Terrain tool |
| **D** | Paint Decorator tool |
| **E** | Eraser tool |
| **I** | Eyedropper tool |
| **G** | Toggle grid overlay |

---

## File Menu

| Action | Description |
|--------|-------------|
| **New** | Create a fresh 20x15 map with test terrains |
| **Open...** | Load a project from a folder |
| **Save** | Save to current project folder (or prompt if none) |
| **Save As...** | Choose a folder to save the project |
| **Quit** | Exit the application |

---

## Project File Format

Projects are saved as a folder containing:

```
my_project/
  project.json        # Map data + terrain/decorator definitions + sprite locations
  atlas_page_0.png    # 4096x4096 sprite atlas page
  atlas_page_1.png    # Additional pages (if needed)
```

A single 4096x4096 atlas page holds up to 576 sprites (24x24 grid of 170x170 cells), which is sufficient for most projects.

---

## Hex Geometry Reference

The editor uses **flat-top hexagons** with **axial coordinates (q, r)**.

```
Flat-top hex:

      ___
     /   \        Edge indices:
    / NW  \          0 = N   (top)
   |   0   |         1 = NE  (upper-right)
   | 5   1 |         2 = SE  (lower-right)
    \ 4 2 /          3 = S   (bottom)
     \_3_/           4 = SW  (lower-left)
                     5 = NW  (upper-left)
```

| Constant | Value |
|----------|-------|
| Sprite canvas | 170 x 170 px |
| Hex radius (center-to-vertex) | 85 px |
| Hex width | 170 px |
| Hex height | ~147 px |
| Column spacing | 127.5 px |
| Row spacing | ~147 px |

---

## Tips for Artists

### Creating Base Sprites

1. Create a 170x170 px document in Photoshop
2. The hex polygon is centered, with ~11.5px padding top and bottom
3. Paint terrain texture filling the hex area
4. Copy and paste into the editor - pixels outside the hex are automatically clipped

### Creating Edge Overlays

1. Use the same 170x170 canvas
2. Paint the terrain bleeding in from ONE edge direction
3. Use an alpha gradient: full opacity at the edge, fading to transparent ~30% into the hex
4. Leave all other areas fully transparent
5. Do NOT clip to hex shape - the overlay needs to extend into the padding zone

### Variant Tips

- Create 2-4 base variants with slight color/texture variation for visual interest
- The editor randomly assigns variants when painting, creating natural-looking terrain
- Click the same terrain on a hex to cycle through variants manually
