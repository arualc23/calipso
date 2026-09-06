use egui_winit::egui;

use crate::game;

pub trait State {
    fn run_frame(&mut self, ui: &mut egui::Ui);
    // fn get_shared_textures(&self) -> SharedTextures;
    fn transition(&mut self) -> Option<Box<dyn State>>;
}


    