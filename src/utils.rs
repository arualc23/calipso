use std::{ops::{Add, AddAssign, Deref, DerefMut, Index, IndexMut}, path::Path, sync::LazyLock, collections::BinaryHeap, cmp::Reverse};

use egui::{ColorImage, Context, TextureHandle};

use crate::{game::{self, LogicalMap}, id::IndexedBy, tile::TileId};

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

///The order is not guaranteed.
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

///This will iterate through pixels left to right, top to bottom (i. e. (0,0), (1,0), (2,0)... ).
pub struct ColorImageHorizontalIter<'a> {
    inner: ColorImageIter<'a>,
}

impl<'a> Iterator for ColorImageHorizontalIter<'a> {
    type Item = &'a egui::Color32;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

///This will iterate trough pixels top to bottom, left to right (i. e. (0,0), (0,1), (0,2)... ).
pub struct ColorImageVerticalIter<'a> {
    inner: &'a ColorImage,
    index: (usize, usize),
    finished: bool,
}

impl<'a> Iterator for ColorImageVerticalIter<'a> {
    type Item = &'a egui::Color32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished { return None; }
        let ret = Some(&self.inner[self.index]);
        self.index.1 += 1;
        if self.index.1 >= self.inner.height() {
            self.index.0 += 1;
            self.index.1 = 0;
        }
        if self.index.0 >= self.inner.width() {
            self.finished = true;
        }

        ret
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

pub fn color_image_to_horizontal_iter(color_image: &ColorImage) -> ColorImageHorizontalIter<'_> {
    ColorImageHorizontalIter {
        inner: ColorImageIter { inner: color_image, index: (0, 0), finished: false }
    }
}

pub fn color_image_to_vertical_iter(color_image: &ColorImage) -> ColorImageVerticalIter<'_> {
    ColorImageVerticalIter { inner: color_image, index: (0, 0), finished: false }
}

#[inline]
pub fn empty_image(size: [usize; 2]) -> ColorImage {
    ColorImage::new(size, vec![game::NULL; size[0] * size[1]])
}



pub(crate) fn has_duplicates<T: PartialEq>(slice: &[T]) -> bool {
    for i in 1..slice.len() {
        if slice[i..].contains(&slice[i - 1]) {
            return true;
        }
    }
    false
}

///Implements Eq and Ord, but panics if comparing NaN.
#[derive(Debug, PartialOrd, Clone, Copy)]
pub(crate) struct Totalf32(f32);

impl PartialEq for Totalf32 {
    fn eq(&self, other: &Self) -> bool {
        assert!(!self.0.is_nan());
        assert!(!other.0.is_nan());
        self.0 == other.0
    }
}
impl Eq for Totalf32 {}
impl Ord for Totalf32 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        assert!(!self.0.is_nan());
        assert!(!other.0.is_nan());
        self.0.partial_cmp(&other.0).unwrap() //because they are not NaN
    }
}

impl From<f32> for Totalf32 {
    fn from(value: f32) -> Self {
        Self(value)
    }
}
impl Into<f32> for Totalf32 {
    fn into(self) -> f32 {
        self.0
    }
}

impl Add for Totalf32 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

#[derive(PartialEq, Eq)]
struct Element(Totalf32, TileId);
impl PartialOrd for Element {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl Ord for Element {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

/// # Returns
/// The map from tile ids to the distance from origin, and the previous tile on the path.
pub(crate) fn dijkstra(lmap: &LogicalMap, tile: TileId) -> IndexedBy<TileId, (f32, TileId)> {
    let mut pool: BinaryHeap<Reverse<Element>> = BinaryHeap::new();
    pool.push(Reverse(Element(0.0.into(), tile)));

    let mut res = IndexedBy::filled(lmap.tiles_count(), (f32::INFINITY.into(), tile));
    // log::info!("lmap length: {}", lmap.tiles_count());
    res[tile] = (0.0, tile);

    while let Some(Reverse(Element(cost, other_tile))) = pool.pop() {
        if cost != res[other_tile].0.into() {
            continue;
        }

        for neighbour in lmap.neighbours(other_tile) {
            let new_cost = cost + lmap.get_tile(other_tile).expect("neighbours only returns valid ids").movement_cost().into();
            if new_cost < res[*neighbour].0.into() {
                res[*neighbour] = (new_cost.into(), other_tile);
                pool.push(Reverse(Element(new_cost, *neighbour)));
            }
        }
    }

    res
}