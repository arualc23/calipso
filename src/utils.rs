use std::sync::Arc;

use egui::{ColorImage, Context, TextureHandle};

use crate::game;

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

/// # Panics
///If improper indexing is attempted
pub fn paste<T: Copy>(
    src: &[T],
    src_width: usize,
    dst: &mut [T],
    dst_width: usize,
    x: usize,
    y: usize,
) {
    let height = src.len() / src_width;

    for row in 0..height {
        let src_start = row * src_width;
        let dst_start = (y + row) * dst_width + x;

        dst[dst_start..dst_start + src_width] 
            .copy_from_slice(&src[src_start..src_start + src_width]);
    }
}


/// # Returns
/// Raw vector, width, height, x_offset, y_offset
pub fn crop<T: PartialEq + Default + Copy>(
    data: &[T],
    width: usize,
) -> Option<(Vec<T>, usize, usize, usize, usize)> {
    let height = data.len() / width;

    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;

    for y in 0..height {
        for x in 0..width {
            if data[y * width + x] != T::default() {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }

    // Nothing but zeros
    if min_x == width {
        return None;
    }

    let new_width = max_x - min_x + 1;
    let new_height = max_y - min_y + 1;

    let mut result = Vec::with_capacity(new_width * new_height);

    for y in min_y..=max_y {
        let start = y * width + min_x;
        let end = start + new_width;
        result.extend_from_slice(&data[start..end]);
    }

    Some((result, new_width, new_height, min_x, min_y))
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