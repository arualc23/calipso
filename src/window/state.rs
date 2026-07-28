use egui_winit::egui;
use std::any::Any;

pub trait State {
    fn process_input(&mut self, gui_channel: Box<dyn Any>);
    fn run_frame(&mut self, ctx: &mut egui::Ui, gui_channel: &mut dyn Any);
    // fn get_shared_textures(&self) -> SharedTextures;
    fn transition(&mut self) -> Option<Box<dyn State>>;
}
