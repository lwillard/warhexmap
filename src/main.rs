pub mod types;
pub mod hex;
pub mod sprites;
pub mod editor;
pub mod project;
pub mod app;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 900.0])
            .with_title("Hex Map & Sprite Editor"),
        ..Default::default()
    };

    eframe::run_native(
        "hex-map-editor",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(app::HexEditorApp::new(cc)))
        }),
    )
}
