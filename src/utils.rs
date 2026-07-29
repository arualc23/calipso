use std::sync::Arc;

pub const ASSETS: &str = "assets";
pub const UV: egui::Rect = egui::Rect {min: egui::Pos2 {x: 0.0, y: 0.0}, max: egui::Pos2 {x: 1.0, y: 1.0}};

#[macro_export]
macro_rules! path {
    ($e: expr) => {
        std::path::Path::new($e)
    };
}


pub fn load_texture_from_filename(filename: &str, context: &egui::Context) -> Result<(Arc<egui::TextureHandle>, egui::ColorImage), image::ImageError> {
    let image = image::ImageReader::open(path!(ASSETS).join(filename))?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
    Ok((Arc::new(context.load_texture(
        filename, 
        color_image.clone(), 
        egui::TextureOptions::LINEAR,
    )), color_image))


}