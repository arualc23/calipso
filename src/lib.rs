use egui_winit::winit::{self, platform::wayland::EventLoopBuilderExtWayland};

pub mod window;

pub fn init(initializer: impl FnOnce() -> Box<dyn window::state::State>, title: String) {
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
    use super::*;
    use log::{debug, error, info};
    #[test]
    pub(crate) fn test1() {
        env_logger::init();
        info!("Staritng test");
        init(|| Box::new(TestState::default()), "Test".to_string());
        info!("After init");
    }

    #[derive(Default)]
    struct TestState {

    }
    impl window::state::State for TestState {
        fn process_input(&mut self, gui_channel: Box<dyn std::any::Any>) {
            
        }
        fn run_frame(&mut self, ui: &mut egui_winit::egui::Ui, gui_channel: &mut dyn std::any::Any) {
            // egui_winit::egui::CentralPanel::default().show_inside(ui, |ui| ui.label("Hello world!"));
            egui_winit::egui::Area::new("fsdfsd".into())
                .anchor(egui_winit::egui::Align2::CENTER_CENTER, (0.0, 0.0))
                .show(ui.ctx(), |ui2| {ui2.label("Hello world!"); ui2.button("dgfhdfgdfg");});
            // info!("test");
        }
        fn transition(&mut self) -> Option<Box<dyn window::state::State>> {
            None
        }
    }



}