use std::{path::Path, sync::{LazyLock}};

use egui::{ColorImage, Context, TextureHandle};

use crate::game;

pub static ASSETS: LazyLock<&Path> = LazyLock::new(|| Path::new("assets"));
pub const UV: egui::Rect = egui::Rect {min: egui::Pos2 {x: 0.0, y: 0.0}, max: egui::Pos2 {x: 1.0, y: 1.0}};

#[macro_export]
macro_rules! path {
    ($e: expr) => {
        std::path::Path::new($e)
    };
}


pub fn load_image_from_path(path: impl AsRef<std::path::Path>) -> Result<ColorImage, image::ImageError> {
    let image = image::ImageReader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

    Ok(color_image)
}

pub enum TextureOptions {
    Exact,
    Smooth,
}

impl Into<egui::TextureOptions> for TextureOptions {
    fn into(self) -> egui::TextureOptions {
        match self {
            Self::Exact => egui::TextureOptions::NEAREST,
            Self::Smooth => egui::TextureOptions::LINEAR,
        }
    }
}

pub fn load_texture_from_image(image: &ColorImage, ctx: &Context, name: impl Into<String>, options: TextureOptions) -> TextureHandle {
    ctx.load_texture(
        name, 
        image.clone(), 
        options.into()
    )
}

pub fn load_texture_from_path(path: impl AsRef<std::path::Path>, ctx: &Context, name: impl Into<String>, options: TextureOptions) -> Result<TextureHandle, image::ImageError> {
    let color_image = load_image_from_path(path)?;
    Ok(load_texture_from_image(&color_image, ctx, name, options))
}

pub struct ColorImageIter<'a> {
    inner: &'a ColorImage,
    index: (usize, usize),
    finished: bool,
}

impl<'a> Iterator for ColorImageIter<'a> {
    type Item = &'a egui::Color32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished { return None; }
        let ret = Some(&self.inner[self.index]);
        self.index.0 += 1;
        if self.index.0 >= self.inner.width() {
            self.index.1 += 1;
            self.index.0 = 0;
        }
        if self.index.1 >= self.inner.height() {
            self.finished = true;
        }

        ret
    }
}

impl<'a> ColorImageIter<'a> {
    pub fn enumerated(self) -> ColorImageEnumeratedIter<'a> {
        ColorImageEnumeratedIter { inner: self }
    }
}

pub struct ColorImageIntoIter {
    inner: ColorImage,
    index: (usize, usize),
    finished: bool,
}

impl Iterator for ColorImageIntoIter {
    type Item = egui::Color32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished { return None; }
        let ret = Some(self.inner[self.index]);
        self.index.0 += 1;
        if self.index.0 >= self.inner.width() {
            self.index.1 += 1;
            self.index.0 = 0;
        }
        if self.index.1 >= self.inner.height() {
            self.finished = true;
        }

        ret
    }
}

pub struct ColorImageEnumeratedIntoIter {
    inner: ColorImageIntoIter,
}

impl Iterator for ColorImageEnumeratedIntoIter {
    type Item = ((usize, usize), egui::Color32);

    fn next(&mut self) -> Option<Self::Item> {
        Some((self.inner.index, self.inner.next()?))
    }
}

pub struct ColorImageEnumeratedIter<'a> {
    inner: ColorImageIter<'a>,
}

impl<'a> Iterator for ColorImageEnumeratedIter<'a> {
    type Item = ((usize, usize), &'a egui::Color32);

    fn next(&mut self) -> Option<Self::Item> {
        Some((self.inner.index, self.inner.next()?))
    }
}

pub fn color_image_to_iter(color_image: &ColorImage) -> ColorImageIter<'_> {
    ColorImageIter { inner: color_image, index: (0, 0), finished: false }
}

#[inline]
pub fn empty_image(size: [usize; 2]) -> ColorImage {
    ColorImage::new(size, vec![game::NULL; size[0] * size[1]])
}