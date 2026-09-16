use egui::{Color32, ColorImage};
use getset::{Getters, CopyGetters};
use std::ops::{Add, Deref, DerefMut, Index, IndexMut};
use crate::{game::{NULL, player}, id::{IdIterator, IndexedBy}, id_derives, map::IdsMap, utils};

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
pub struct Tile {
    #[getset(get_copy = "pub")]
    id: TileId,
    raw_texture: ColorImage,
    #[getset(get_copy = "pub")]
    movement_cost: f32,
    pub controller: player::PlayerId,
}

impl Tile {
    pub(crate) fn paste_onto_canvas(&self, canvas: &mut ColorImage, player_color_getter: impl FnOnce(player::PlayerId) -> Color32) {

        let player_tint = player_color_getter(self.controller);

        let iter = utils::color_image_to_iter(&self.raw_texture).enumerated();
        for (pos, &color) in iter {
            if color == NULL { continue; }
            canvas[pos] = color.blend(player_tint);
        }
    }

    pub(crate) fn paste_from_image(&mut self, image: impl AsRef<ColorImage>, ids_map: impl AsRef<IdsMap>) {
        for (pos, pixel) in utils::color_image_to_iter(image.as_ref()).enumerated().filter(|(pos, _)| self.id == ids_map.as_ref()[*pos].into()) {
            self.raw_texture[pos] = *pixel;
        }
    }

    pub(crate) fn empty(id: TileId, size: [usize; 2], movement_cost: f32, controller: player::PlayerId) -> Self {
        Self {
            id,
            raw_texture: utils::empty_image(size),
            movement_cost,
            controller
        }
    }
}

// pub(crate) struct TilesContainer {
//     pub(crate) inner: IndexedBy<TileId, Tile>
// }

// impl TilesContainer {
//     // pub fn new(length: usize, size: [usize; 2],) -> Self {
//     //     let inner = unsafe {
//     //         IndexedBy::new(Vec::from_fn(length, |id| Tile::empty(TileId::from(id), size, 1.0)))
//     //     };
//     //     Self { inner }
//     // }

//     pub fn new(length: usize, size: [usize; 2], movement_costs: &IndexedBy<TileId, f32>) -> Self {
//         let inner = unsafe {
//             IndexedBy::new(Vec::from_fn(length, |id| Tile::empty(TileId::from(id), size, movement_costs[id.into()])))
//         };
//         Self { inner }
//     }

//     // pub fn iter_ids(&self) -> IdIterator<TileId> {
//     //     self.inner.iter_ids()
//     // }

//     // pub fn iter(&self) -> core::slice::Iter<'_, Tile> {
//     //     self.inner.iter()
//     // }

//     // pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, Tile> {
//     //     self.inner.iter_mut()
//     // }
// }

// impl Deref for TilesContainer {
//     type Target = IndexedBy<TileId, Tile>;

//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

// impl DerefMut for TilesContainer {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner
//     }
// }

// // impl Index<TileId> for TilesContainer {
// //     type Output = Tile;
// //     fn index(&self, index: TileId) -> &Self::Output {
// //         &self.inner[index]
// //     }
// // }
