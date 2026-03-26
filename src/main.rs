mod hex_math;
mod model;
mod renderer;
mod editor;
mod ui;
mod app;

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

struct WinHandler {
    window: Option<Arc<Window>>,
    app: Option<app::App>,
}

impl ApplicationHandler for WinHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = WindowAttributes::default()
                .with_title("Hex Terrain Map Editor")
                .with_inner_size(winit::dpi::LogicalSize::new(1400, 900));
            let window = Arc::new(event_loop.create_window(attrs).unwrap());
            self.app = Some(app::App::new(window.clone()));
            self.window = Some(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        let Some(app) = &mut self.app else { return };
        let Some(window) = &self.window else { return };

        match &event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                app.resize(*size);
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                app.update();
                app.render(window);
            }
            _ => {
                app.handle_input(window, &event);
                window.request_redraw();
            }
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut handler = WinHandler {
        window: None,
        app: None,
    };
    event_loop.run_app(&mut handler).unwrap();
}
