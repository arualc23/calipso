#![feature(vec_from_fn)]
#![feature(trait_alias)]

use egui_winit::winit::{self, platform::wayland::EventLoopBuilderExtWayland};

pub mod window;
pub mod map;
pub mod utils;
pub mod game;
pub mod unit_display;
pub mod tile;

pub mod consts;

pub(crate) use window::CLOSING_REQUESTED;
pub use consts::*;

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

use crate::{unit_display::UnitDisplay};

    use super::*;
#[allow(unused_imports)]
    use log::{debug, error, info, warn};

    #[test]
    pub(crate) fn test1() {
        env_logger::init();
        info!("Staritng test");
        init(|renderer| {
            let (map, lmap) = map::load_map("test1", renderer.ctx().clone()).unwrap();
            Box::new(TestState::new(map, lmap))
        }, "Test".to_string());
        info!("After init");
    }

    struct TestState {
        map: map::Map,
        units: Vec<unit_display::UnitDisplay>,
    }

    impl window::state::State for TestState {
        fn run_frame(&mut self, ui: &mut egui_winit::egui::Ui) {

            let painter = ui.layer_painter(*BACKGROUND_LAYER);
            self.map.run_frame(ui, Some(&painter), &self.units);
        }
        fn transition(&mut self) -> Option<Box<dyn window::state::State>> {
            None
        }
    }

    impl TestState {
        pub fn new(map: map::Map, logical_map: game::LogicalMap) -> Self {
            let unit_texture = utils::load_texture_from_path("assets/textures/unit.png", &map.ctx(), "unit", utils::TextureOptions::Smooth).unwrap();

            let send = game::interface::init_game_loop(|(_input, _msg): &(_, ())| info!("Hello world!"), logical_map, map.get_raw_image());
            let _ = Box::leak(Box::new(send)); //so that main loop doesn't exit immediately

            Self { map, units: vec![UnitDisplay::new(0.into(), unit_texture)] }
        }
        
    }



}