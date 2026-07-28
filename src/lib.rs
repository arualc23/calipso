use egui_winit::winit::{self, platform::wayland::EventLoopBuilderExtWayland};

pub mod window;
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
    use std::{ops::Index, sync::Arc};

use super::*;
    use egui::{Color32, ColorImage, Pos2, Rect, TextureHandle, Vec2};
use log::{debug, error, info};

    const UV: Rect = Rect {min: Pos2 {x: 0.0, y: 0.0}, max: Pos2 {x: 1.0, y: 1.0}};
    const SCROLL_SCALE: f32 = 500.0;
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

    struct Map {
        texture: Arc<TextureHandle>,
        rect: Rect,
        raw: ColorImage,
        starting_rect: Rect,
        starting_diag: f32,
    }

    impl Map {
        fn update(&mut self, camera_coords: Vec2, cursor_coords: Vec2, scroll: f32) {
            Self::update_point(&mut self.rect.min, cursor_coords, scroll, camera_coords);
            Self::update_point(&mut self.rect.max, cursor_coords, scroll, camera_coords);

        }

        fn update_point(point: &mut Pos2, cursor_coords: Vec2, scroll: f32, camera_coords: Vec2) {
            *point += (*point - camera_coords - cursor_coords).to_vec2() * scroll/SCROLL_SCALE;
        }

        fn get_color(&self, x: isize, y: isize) -> Color32 {
            let (Ok(x), Ok(y)) = (
                usize::try_from(x),
                usize::try_from(y)
            ) else {return Color32::PLACEHOLDER};
            *(self.raw.get((x, y)).unwrap_or(&Color32::PLACEHOLDER))
        }

        fn current_to_starting_coords(&self, pos: Pos2) -> Pos2 {
            self.starting_rect.min + (pos - self.rect.min)/self.rect.size().length()*self.starting_diag
        }

        fn new(texture: Arc<TextureHandle>, raw: ColorImage, starting_rect: Rect, ) -> Self {
            let rect = starting_rect;
            let starting_diag = rect.size().length();
            // dbg!(raw.pixels.iter().any(|&val| val != Color32::BLACK));
            dbg!(raw.size);
            Self {
                texture,
                rect,
                raw,
                starting_rect,
                starting_diag
            }
        }
    }

    // #[derive(Default)]
    struct TestState {
        map: Map
        
    }
    impl window::state::State for TestState {
        fn process_input(&mut self, gui_channel: Box<dyn std::any::Any>) {
            
        }
        fn run_frame(&mut self, ui: &mut egui_winit::egui::Ui, gui_channel: &mut dyn std::any::Any) {
            // egui_winit::egui::CentralPanel::default().show_inside(ui, |ui| ui.label("Hello world!"));
            let painter = ui.debug_painter();
            painter.image(self.map.texture.id(), self.map.rect, UV, Color32::WHITE);
            // egui::Area::new("fsdfsd".into())
            //     .anchor(egui_winit::egui::Align2::CENTER_CENTER, (0.0, 0.0))
            //     .show(ui.ctx(), |ui2| {ui2.label("Hello world!"); ui2.button("dgfhdfgdfg");});

            let camera_coords = Vec2::ZERO;
            let cursor_coords = ui.input(|i| i.pointer.latest_pos().unwrap_or_default());
            let scroll = ui.input(|i| i.smooth_scroll_delta().y);

            if ui.input(|i| i.pointer.primary_clicked()) {
                let translated_pos = self.map.current_to_starting_coords(cursor_coords);
                info!("{}", translated_pos);
                info!("{:?}", self.map.get_color(translated_pos.x as isize, translated_pos.y as isize));
            }

            self.map.update(camera_coords, cursor_coords.to_vec2(), scroll);
            // *gui_channel = *data;
            // info!("test");
        }
        fn transition(&mut self) -> Option<Box<dyn window::state::State>> {
            None
        }
    }

    impl TestState {
        pub fn new(map_texture: Arc<TextureHandle>, raw: ColorImage) -> Self {
            let starting_rect = Rect::from_two_pos(Pos2::ZERO, (raw.size[0] as f32, raw.size[1] as f32).into());
            Self { map: Map::new(map_texture, raw, starting_rect) }
        }
        
    }
    struct GUIChannel {
        camera_coords: Vec2,
        cursor_coords: Vec2,
        scroll: f32,
    }

    trait SafeIndex<Idx>: Index<Idx> {
        fn get(&self, index: Idx) -> Option<&Color32>;
    }
    impl SafeIndex<(usize, usize)> for ColorImage {
        fn get(&self, index: (usize, usize)) -> Option<&Color32> {
            if index.0 < self.size[0] && index.1 < self.size[1] {
                Some(&self[index])
            } else {
                None
            }
        }
    }



}