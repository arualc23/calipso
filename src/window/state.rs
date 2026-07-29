use egui_winit::egui;

pub trait State {
    fn run_frame(&mut self, ctx: &mut egui::Ui);
    // fn get_shared_textures(&self) -> SharedTextures;
    fn transition(&mut self) -> Option<Box<dyn State>>;
}
