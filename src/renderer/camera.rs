#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub hex_size: f32,
    pub grid_offset_q: i32,
    pub grid_offset_r: i32,
    pub _pad: f32,
}

pub struct Camera {
    pub view_x: f32,
    pub view_y: f32,
    pub zoom: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            view_x: 0.0,
            view_y: 0.0,
            zoom: 1.0,
            min_zoom: 0.25,
            max_zoom: 16.0,
        }
    }

    /// Builds an orthographic projection uniform.
    ///
    /// The visible world region spans from `(view_x, view_y)` to
    /// `(view_x + viewport_w / zoom, view_y + viewport_h / zoom)`.
    pub fn build_uniform(
        &self,
        viewport_w: f32,
        viewport_h: f32,
        hex_size: f32,
        grid_offset: (i32, i32),
    ) -> CameraUniform {
        let left = self.view_x;
        let right = self.view_x + viewport_w / self.zoom;
        let top = self.view_y;
        let bottom = self.view_y + viewport_h / self.zoom;

        // Orthographic projection matrix mapping [left..right, top..bottom] to NDC [-1..1]
        let sx = 2.0 / (right - left);
        let sy = 2.0 / (top - bottom); // flip y so +y is up in NDC but down on screen
        let tx = -(right + left) / (right - left);
        let ty = -(top + bottom) / (top - bottom);

        // Column-major 4x4 matrix
        let view_proj = [
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [tx, ty, 0.0, 1.0],
        ];

        CameraUniform {
            view_proj,
            hex_size,
            grid_offset_q: grid_offset.0,
            grid_offset_r: grid_offset.1,
            _pad: 0.0,
        }
    }

    /// Converts screen coordinates (pixels from top-left) to world coordinates.
    pub fn screen_to_world(&self, sx: f32, sy: f32, vp_w: f32, vp_h: f32) -> (f32, f32) {
        let world_x = self.view_x + sx / self.zoom;
        let world_y = self.view_y + sy / self.zoom;
        let _ = (vp_w, vp_h); // available for future clamping
        (world_x, world_y)
    }

    /// Zoom centered on the cursor position.
    ///
    /// `delta` > 0 zooms in, `delta` < 0 zooms out.
    pub fn zoom_at(
        &mut self,
        screen_x: f32,
        screen_y: f32,
        vp_w: f32,
        vp_h: f32,
        delta: f32,
    ) {
        let (world_x, world_y) = self.screen_to_world(screen_x, screen_y, vp_w, vp_h);

        let factor = 1.1_f32.powf(delta);
        self.zoom = (self.zoom * factor).clamp(self.min_zoom, self.max_zoom);

        // Adjust view so the world point under cursor stays at the same screen pixel
        self.view_x = world_x - screen_x / self.zoom;
        self.view_y = world_y - screen_y / self.zoom;
    }

    /// Pan by a screen-pixel offset.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.view_x -= dx / self.zoom;
        self.view_y -= dy / self.zoom;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
