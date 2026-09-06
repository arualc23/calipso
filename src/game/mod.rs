use std::{ops::{Index}, sync::{Arc, Mutex}};

use egui::{Color32, ColorImage};

use crate::utils;

pub const NULL: Color32 = Color32::from_rgba_premultiplied(0, 0 ,0, 0);

pub struct PlayerId;
pub struct Player {
    id: PlayerId
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct TileId {
    inner: u32
} 

impl TileId {
    pub fn new(inner: u32) -> Self {
        Self {
            inner
        }
    }
}

impl From<Color32> for TileId {
    fn from(value: Color32) -> Self {
        let inner: u32 = (value.r() as u32) << 16 +
                         (value.g() as u32) << 8 +
                         (value.b() as u32) << 0;
        Self { inner }
    }
}

impl From<usize> for TileId {
    fn from(value: usize) -> Self {
        Self { inner: value as u32 }
    }
}

pub(crate) struct Tile {
    id: TileId,
    raw_texture: ColorImage,
}

impl Tile {
    fn paste_onto_canvas(&self, canvas: &mut ColorImage) {

        let iter = utils::color_image_to_iter(&self.raw_texture).enumerated();
        for (pos, &color) in iter {
            canvas[pos] = canvas[pos].blend(color);
        }
    }

    fn empty(id: TileId, size: [usize; 2]) -> Self {
        Self {
            id,
            raw_texture: utils::empty_image(size)
        }
    }
}

pub(crate) struct TilesContainer {
    inner: Vec<Tile>
}

impl TilesContainer {
    pub fn new(length: usize, size: [usize; 2]) -> Self {
        let inner = Vec::from_fn(length, |id| Tile::empty(TileId::from(id), size));
        Self { inner }
    }

    pub fn iter_ids(&self) -> Box<dyn Iterator<Item = TileId>> {
        Box::new(
            (0..self.inner.len()).map(|item| TileId::from(item))
        )
    }

    pub fn iter(&self) -> core::slice::Iter<'_, Tile> {
        self.inner.iter()
    }
}

impl Index<TileId> for TilesContainer {
    type Output = Tile;
    fn index(&self, index: TileId) -> &Self::Output {
        &self.inner[index.inner as usize]
    }
}

pub struct LogicalMap {
    map: TilesContainer,
    real_image: ColorImage,
    updated: bool,
}

impl LogicalMap {
    fn export_real_image(&self) -> &ColorImage {
        &self.real_image
    }
    fn update_tile_texture(&mut self, tile_id: TileId) {
        self.map[tile_id].paste_onto_canvas(&mut self.real_image);
    }

    pub(crate) fn new(map: TilesContainer, size: [usize; 2]) -> Self {
        let mut real_image = utils::empty_image(size);
        for tile in map.iter() {
            tile.paste_onto_canvas(&mut real_image);
        }

        Self {
            map,
            real_image,
            updated: true,
        }
    }
}

pub trait GameLoop = FnMut() + Send + 'static;


pub(crate) fn game_loop(mut body: impl GameLoop, mut logical_map: LogicalMap, real_image: Arc<Mutex<ColorImage>>) {
    loop {
        if crate::CLOSING_REQUESTED.load(std::sync::atomic::Ordering::Acquire) {
            break;
        }
        body();

        if logical_map.updated {
            let mut image = real_image.lock().unwrap();
            *image = logical_map.real_image.clone();
            logical_map.updated = false;
        }
    }
}


pub fn init_game_loop(game_loop_body: impl GameLoop, logical_map: LogicalMap, real_image: Arc<Mutex<ColorImage>>) {
    let _ = std::thread::spawn(move || game_loop(game_loop_body, logical_map, real_image));
}