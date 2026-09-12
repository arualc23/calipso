use egui::{Color32, ColorImage};
use getset::{Getters, CopyGetters};
use std::ops::{Add, Deref, DerefMut, Index, IndexMut};
use crate::{id_derives, map::IdsMap, utils, id::{IdIterator, IndexedBy}};

id_derives!{pub TileId}

impl From<Color32> for TileId {
    fn from(value: Color32) -> Self {
        let inner: u32 = ((value.r() as u32) << 24 ) |
                         ((value.g() as u32) << 16 ) |
                         ((value.b() as u32) << 8  ) |
                         ((value.a() as u32));
        Self(inner)
    }
}

impl Into<Color32> for TileId {
    fn into(self) -> Color32 {
        Color32::from_rgba_premultiplied(
            (self.0 >> 24) as u8,
            (self.0 >> 16) as u8,
            (self.0 >>  8) as u8,
            (self.0 >>  0) as u8,
        )
    }
}

#[derive(Debug, Clone, Getters, CopyGetters)]
pub(crate) struct Tile {
    #[getset(get_copy = "pub")]
    id: TileId,
    raw_texture: ColorImage,
    #[getset(get_copy = "pub")]
    movement_cost: f32,
}

impl Tile {
    pub(crate) fn paste_onto_canvas(&self, canvas: &mut ColorImage) {

        let iter = utils::color_image_to_iter(&self.raw_texture).enumerated();
        for (pos, &color) in iter {
            // if color == Color32::TRANSPARENT && canvas[pos] != color { log::info!("what"); }
            if color == Color32::TRANSPARENT { continue; }
            canvas[pos] = color;
        }
    }

    pub(crate) fn paste_from_image(&mut self, image: impl AsRef<ColorImage>, ids_map: impl AsRef<IdsMap>) {
        for (pos, pixel) in utils::color_image_to_iter(image.as_ref()).enumerated().filter(|(pos, _)| self.id == ids_map.as_ref()[*pos].into()) {
            self.raw_texture[pos] = *pixel;
        }
    }

    fn empty(id: TileId, size: [usize; 2], movement_cost: f32) -> Self {
        Self {
            id,
            raw_texture: utils::empty_image(size),
            movement_cost
        }
    }
}

pub(crate) struct TilesContainer {
    pub(crate) inner: IndexedBy<TileId, Tile>
}

impl TilesContainer {
    pub fn new(length: usize, size: [usize; 2]) -> Self {
        let inner = unsafe {
            IndexedBy::new(Vec::from_fn(length, |id| Tile::empty(TileId::from(id), size, 1.0)))
        };
        Self { inner }
    }

    // pub fn iter_ids(&self) -> IdIterator<TileId> {
    //     self.inner.iter_ids()
    // }

    // pub fn iter(&self) -> core::slice::Iter<'_, Tile> {
    //     self.inner.iter()
    // }

    // pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, Tile> {
    //     self.inner.iter_mut()
    // }
}

impl Deref for TilesContainer {
    type Target = IndexedBy<TileId, Tile>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TilesContainer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// impl Index<TileId> for TilesContainer {
//     type Output = Tile;
//     fn index(&self, index: TileId) -> &Self::Output {
//         &self.inner[index]
//     }
// }
