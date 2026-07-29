use egui_winit::winit::{self, platform::wayland::EventLoopBuilderExtWayland};

pub mod window;
pub mod map;
mod utils;

pub fn init(initializer: impl FnOnce(&window::EguiRenderer) -> Box<dyn window::state::State>, title: String) {
    #[cfg(test)]
    let event_loop = winit::event_loop::EventLoop::builder().with_any_thread(true).build().unwrap();
    #[cfg(not(test))]
    let event_loop = winit::event_loop::EventLoop::builder().with_any_thread(false).build().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = window::ControlFlow::new(initializer, title);

    event_loop.run_app(&mut app).unwrap();
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc};

    use super::*;
    use egui::{Color32, ColorImage, Pos2, Rect, TextureHandle, Vec2};
    #[allow(unused_imports)]
    use log::{debug, error, info, warn};

    #[test]
    pub(crate) fn test1() {
        env_logger::init();
        info!("Staritng test");
        init(|renderer| {
            let (map_texture, raw) = utils::load_texture_from_filename("test1.png", renderer.ctx()).unwrap();
            Box::new(TestState::new(map_texture, raw))
        }, "Test".to_string());
        info!("After init");
    }

    struct TestState {
        map: map::Map
    }

    impl window::state::State for TestState {
        fn run_frame(&mut self, ui: &mut egui_winit::egui::Ui) {

            let painter = ui.debug_painter();
            painter.image(self.map.texture.id(), self.map.rect, utils::UV, Color32::WHITE);

            let camera_coords = Vec2::ZERO;
            let cursor_coords = ui.input(|i| i.pointer.latest_pos().unwrap_or_default());
            let scroll = ui.input(|i| i.smooth_scroll_delta().y);

            if ui.input(|i| i.pointer.primary_clicked()) {
                debug!("{:?}", self.map.handle_click(cursor_coords));
            }

            self.map.update(camera_coords, cursor_coords.to_vec2(), scroll);

        }
        fn transition(&mut self) -> Option<Box<dyn window::state::State>> {
            None
        }
    }

    impl TestState {
        pub fn new(map_texture: Arc<TextureHandle>, raw: ColorImage) -> Self {
            let starting_rect = Rect::from_two_pos(Pos2::ZERO, (raw.size[0] as f32, raw.size[1] as f32).into());
            Self { map: map::Map::new(map_texture, raw, starting_rect) }
        }
        
    }



}