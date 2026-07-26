use egui_winit::winit;

pub mod window;

pub fn run() {
    env_logger::init();
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = window::ControlFlow::new(|| Box::new(window::state::NoopState::default()), "Absolutne kino".to_string());

    event_loop.run_app(&mut app).unwrap();
}