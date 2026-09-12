use egui_winit::egui;
pub trait State {
    fn run_frame(&mut self, ui: &mut egui::Ui);
    fn transition(&mut self) -> Option<Box<dyn State>>;
}


    