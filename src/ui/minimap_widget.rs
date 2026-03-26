use crate::renderer::camera::Camera;

pub struct MinimapWidget;

impl MinimapWidget {
    /// Show the minimap texture. Clicking on it pans the camera to that world position.
    ///
    /// `world_bounds` is (min_x, min_y, max_x, max_y) of the full map in world coords.
    pub fn show(
        ui: &mut egui::Ui,
        texture_id: egui::TextureId,
        camera: &mut Camera,
        world_bounds: (f32, f32, f32, f32),
    ) {
        ui.heading("Minimap");
        ui.add_space(4.0);

        let minimap_size = egui::vec2(180.0, 120.0);
        let img = egui::Image::new(egui::load::SizedTexture::new(texture_id, minimap_size));
        let response = ui.add(img);

        if response.clicked() || response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let rect = response.rect;
                // Normalize click position within the minimap widget [0..1]
                let nx = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                let ny = ((pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0);

                let (min_x, min_y, max_x, max_y) = world_bounds;
                let world_x = min_x + nx * (max_x - min_x);
                let world_y = min_y + ny * (max_y - min_y);

                camera.view_x = world_x;
                camera.view_y = world_y;
            }
        }
    }
}
