# Changelog

All notable changes to the Hex Map & Sprite Editor project.

## [0.1.0] - 2026-03-26

### Complete Rewrite: eframe/egui with Sprite Atlas Rendering

Replaced the raw wgpu + custom shader approach with eframe/egui for all rendering,
and switched from procedural texture splatting to a sprite-based edge overlay system.

### Added

- **Dual-mode editor**: Sprite Editor and Map Editor tabs
- **Sprite Editor panel**
  - Terrain type management (add, remove, select)
  - Decorator type management with categories (Natural, Settlement, Road, River, Custom)
  - 7x4 sprite slot grid per terrain (1 row base variants + 6 rows edge overlays, 4 variants each)
  - Clipboard paste from Photoshop or any image editor (Ctrl+V workflow)
  - Auto-resize pasted images to 170x170
  - Hex polygon clipping for base sprites (edge overlays left unclipped for seamless bleed)
  - Clear slot support
  - New terrains start with a white hex fill
- **Map Editor panel**
  - Scrollable, zoomable hex map viewport
  - Pan via middle-mouse or right-click drag
  - Zoom via scroll wheel (0.1x - 5.0x, cursor-aware)
  - Tool palette: Paint Terrain, Paint Decorator, Eraser, Eyedropper, Label
  - Terrain palette with selectable terrain types
  - Decorator palette with selectable decorator types
  - Adjustable brush size (1-5 hex rings)
  - Toggleable grid overlay and coordinate labels
  - Hover info in status bar (hex coords + terrain name)
- **Sprite atlas system**
  - Multi-page 4096x4096 atlas with 170x170 sprite cells (24x24 grid per page)
  - Automatic page allocation and GPU texture upload
  - Three lookup tables: base sprites, edge overlays, decorators
- **Rendering pipeline**
  - Layered per-hex rendering: base sprite, edge overlays (priority-based), decorators, grid, coords
  - Terrain priority system controls which terrain's edges bleed on top
  - Viewport culling (only visible hexes rendered)
- **Undo/redo system** with per-cell state snapshots
- **Project save/load**
  - Saves as folder: project.json + atlas page PNGs
  - Full round-trip serialization of map, terrain/decorator defs, and sprite locations
- **Keyboard shortcuts**
  - Ctrl+Z / Ctrl+Shift+Z: Undo / Redo
  - Ctrl+S: Save
  - B: Paint Terrain tool
  - E: Eraser tool
  - I: Eyedropper tool
  - G: Toggle grid
- **4 test terrains** generated at startup: Forest, Plains, Water, Mountain
  - Each with 2 base variants and 6 edge overlays
  - Color-coded with priority ordering

### Removed

- Raw wgpu render pipelines and custom WGSL shaders (terrain, grid_overlay, path, decorator)
- Procedural Perlin noise terrain textures
- winit ApplicationHandler event loop
- Per-pixel IDW texture splatting
- Old module structure: model/, renderer/, shaders/, ui/

### Changed

- Dependencies: replaced wgpu/winit/egui-wgpu/egui-winit/noise/bytemuck/pollster with eframe/egui_extras/arboard/rand
- Hex geometry moved from src/hex_math.rs to src/hex/geometry.rs
- Editor tools consolidated into src/editor/tools.rs (was 7 separate files)
- Map data moved from src/model/ to src/hex/map.rs

---

## [0.0.1] - 2026-03-25

### Initial Prototype

- Hex terrain map editor with Rust/wgpu
- Raw wgpu render pipelines with custom WGSL shaders
- Procedural terrain textures via Perlin noise (28 layers)
- IDW texture splatting in GPU fragment shader
- Hillshade rendering with screen-space derivatives
- egui UI panels via egui-winit + egui-wgpu integration
- winit 0.30 event loop with ApplicationHandler trait
