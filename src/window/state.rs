use egui_winit::egui;
use std::any::Any;

pub trait State {
    fn process_input(&mut self, gui_channel: Box<dyn Any>);
    fn run_frame(&mut self, ctx: &egui::Context, gui_channel: &mut dyn Any);
    // fn get_shared_textures(&self) -> SharedTextures;
    fn transition(&mut self) -> Option<Box<dyn State>>;

}

#[derive(Default)]
pub struct NoopState {

}

impl State for NoopState {
    fn process_input(&mut self, gui_channel: Box<dyn Any>) {
        
    }

    fn run_frame(&mut self, ctx: &egui::Context, gui_channel: &mut dyn Any) {
        
    }

    fn transition(&mut self) -> Option<Box<dyn State>> {
        None
    }
}