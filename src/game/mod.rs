use std::ops::{Index, IndexMut};

use egui::{Color32, ColorImage, Vec2, vec2};
use egui_wgpu::wgpu::wgc::command::DrawKind::MultiDrawIndirectCount;

use crate::utils;

mod game_state;

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

/// Used when constructing tiles from source image. 
// struct TileBuilder {
//     id: TileId,
//     raw_texture: ColorImage,
//     controller: Option<PlayerId>,
// }
// impl TileBuilder {
//     /// size is the size of the entire map!
//     fn new(id: TileId, controller: Option<PlayerId>, size: [usize; 2]) -> Self {
//         Self {
//             id, 
//             controller,
//             raw_texture: ColorImage::filled(size, NULL)
//         }
//     }

//     fn paint(&mut self, pos: (usize, usize), color: Color32) {
//         self.raw_texture[pos] = color
//     }

//     fn build(self) -> Tile {
//         let (raw, width, height, x_offset, y_offset) = utils::crop(self.raw_texture.as_raw(), self.raw_texture.width()).expect("testing");
//         let new_image = ColorImage::new([width, height], raw.chunks_exact(4).map(|item| Color32::from_rgba_premultiplied(item[0], item[1], item[2], item[3])).collect());

//         Tile {
//             id: self.id,
//             controller: self.controller,
//             raw_texture: new_image,
//             offset: [x_offset, y_offset]
//         }
//     }
// }

// pub struct Tile {
//     id: TileId,
//     controller: Option<PlayerId>,
//     raw_texture: ColorImage,
//     offset: [usize; 2],
// }
// impl Tile {
//     fn paste_onto_canvas(&self, canvas: &mut ColorImage) {
//         let dst_width = canvas.size[0];
//         utils::paste(self.raw_texture.as_raw(), self.raw_texture.size[0],
//             canvas.as_raw_mut(), dst_width, self.offset[0], self.offset[1])
//     }
// }

// pub(crate) struct TilesContainerBuilder<'a> {
//     inner: Vec<TileBuilder>,
//     raw_image: &'a ColorImage,
// }

// impl<'a> TilesContainerBuilder<'a> {
//     pub fn new(length: usize, size: [usize; 2], raw_image: &'a ColorImage) -> Self {
//         let inner = Vec::from_fn(length, |id| TileBuilder::new(id.into(), None, size));
//         Self { inner, raw_image }
//     }

//     pub fn paint(&mut self, pos: (usize, usize), id: TileId) {
//         let color = self.raw_image[pos];
//         self.get_mut(id).paint(pos, color);
//     }

//     fn get_mut(&mut self, index: TileId) -> &mut TileBuilder {
//         &mut self.inner[index.inner as usize]
//     }

//     pub fn build(self) -> TilesContainer {
//         TilesContainer {
//             inner: self.inner.into_iter().map(|item| item.build()).collect()
//         }
//     }
// }

pub(crate) struct Tile {
    id: TileId,
    raw_texture: ColorImage,
}

impl Tile {
    fn paste_onto_canvas(&self, canvas: &mut ColorImage) {
        // let dst_width = canvas.size[0];
        // utils::paste(self.raw_texture.as_raw(), self.raw_texture.size[0],
        //     canvas.as_raw_mut(), dst_width, self.offset[0], self.offset[1])

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
}

impl LogicalMap {
    fn export_real_image(&self) -> &ColorImage {
        &self.real_image
    }
    fn update_tile_texture(&mut self, tile_id: TileId) {
        self.map[tile_id].paste_onto_canvas(&mut self.real_image);
    }

    pub fn new(map: TilesContainer, size: [usize; 2]) -> Self {
        let mut real_image = utils::empty_image(size);
        for tile in map.iter() {
            tile.paste_onto_canvas(&mut real_image);
        }

        Self {
            map,
            real_image
        }
    }
}

//The plan is the following: inside the ids_map file we have the raw division of the tiles. Inside the visual file we have the base texture of the entire map, that is split into the tiles according to the ids_map (if there is a size mismatch etc it is an error). Tiles store a rectagle containing their own base texture, everything else transparent, as well as the offset from the global origin of this rectangle, and the size of this rectangle. TIles also store additional graphics (tint etc.), and have a method returning their final texture. real_image is created from pasting all the tiles on a blank canvas, and then updating this. Therefore, real_image is part of the state.