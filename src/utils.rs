use std::sync::Arc;

use egui::{ColorImage, Context, TextureHandle};

pub const ASSETS: &str = "assets";
pub const UV: egui::Rect = egui::Rect {min: egui::Pos2 {x: 0.0, y: 0.0}, max: egui::Pos2 {x: 1.0, y: 1.0}};

#[macro_export]
macro_rules! path {
    ($e: expr) => {
        std::path::Path::new($e)
    };
}


pub fn load_texture_from_filename(filename: &str, ctx: &Context) -> Result<(Arc<egui::TextureHandle>, egui::ColorImage), image::ImageError> {
    let color_image = load_image_from_path(path!(ASSETS).join(filename))?;
    Ok((load_texture_from_image(&color_image, ctx, filename), color_image))


}

pub fn load_image_from_path(path: impl AsRef<std::path::Path>) -> Result<ColorImage, image::ImageError> {
    let image = image::ImageReader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

    Ok(color_image)
}

pub fn load_texture_from_image(image: &ColorImage, ctx: &Context, name: impl Into<String>) -> Arc<TextureHandle> {
    Arc::new(ctx.load_texture(
        name, 
        image.clone(), 
        egui::TextureOptions::LINEAR,
    ))
}

pub fn load_texture_from_path(path: impl AsRef<std::path::Path>, ctx: &Context, name: impl Into<String>) -> Result<Arc<TextureHandle>, image::ImageError> {
    let color_image = load_image_from_path(path)?;
    Ok(load_texture_from_image(&color_image, ctx, name))
}