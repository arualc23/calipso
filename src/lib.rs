#![feature(vec_from_fn)]
#![feature(trait_alias)]

use egui_winit::winit::{self, platform::wayland::EventLoopBuilderExtWayland};

pub mod window;
pub mod map;
mod utils;
mod game;

pub use game::init_game_loop;

pub(crate) use window::CLOSING_REQUESTED;

pub fn init(initializer: impl FnOnce(&window::EguiRenderer) -> Box<dyn window::state::State>, title: String) {
    #[cfg(test)]
    let event_loop = winit::event_loop::EventLoop::builder().with_any_thread(true).build().unwrap();
    #[cfg(not(test))]
    let event_loop = winit::event_loop::EventLoop::builder().with_any_thread(false).build().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    // log::info!("Closing requested: {:?}", CLOSING_REQUESTED);

    let mut app = window::ControlFlow::new(initializer, title);

    event_loop.run_app(&mut app).unwrap();
}

#[cfg(test)]
mod tests {

use crate::map::MapDisplay;

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
        map: map::Map
    }

    impl MapDisplay for TestState {
        fn map(&mut self) -> &mut map::Map {
            &mut self.map
        }
    }

    impl window::state::State for TestState {
        fn run_frame(&mut self, ui: &mut egui_winit::egui::Ui) {

            let painter = ui.debug_painter();
            self.paint_map(ui, &painter);

        }
        fn transition(&mut self) -> Option<Box<dyn window::state::State>> {
            None
        }
    }

    impl TestState {
        pub fn new(map: map::Map, logical_map: game::LogicalMap) -> Self {
            let send = game::init_game_loop(|_input| info!("Hello world!"), logical_map, map.get_raw_image());
            Self { map }
        }
        
    }



}