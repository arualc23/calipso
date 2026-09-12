use egui::{Color32, ColorImage};
use std::ops::{Add, Index, IndexMut};
use crate::{id_derives, map::IdsMap, utils, id::{IdIterator, IndexedBy}};

id_derives!{pub TileId}

// #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
// pub struct TileId {
//     inner: u32
// } 

// impl std::fmt::Display for TileId {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.inner)
//     }
// }

// impl TileId {
//     pub fn new(inner: u32) -> Self {
//         Self {
//             inner
//         }
//     }
// }

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

// impl Into<usize> for TileId {
//     fn into(self) -> usize {
//         self.inner as usize
//     }
// }

// impl From<usize> for TileId {
//     fn from(value: usize) -> Self {
//         Self { inner: value as u32 }
//     }
// }

// impl Add<u32> for TileId {
//     type Output = TileId;
//     fn add(self, rhs: u32) -> Self::Output {
//         Self {inner: self.inner + rhs}
//     }
// }

///end excluded, start included
// pub struct TileIdIter {
//     current: TileId,
//     end: Option<TileId>,
// }

// impl TileIdIter {
//     pub fn new(start: impl Into<TileId>, end: impl Into<Option<TileId>>) -> Self {
//         Self {
//             current: start.into(),
//             end: end.into()
//         }
//     }
// }

// impl Iterator for TileIdIter {
//     type Item = TileId;

//     fn next(&mut self) -> Option<Self::Item> {
//         if let Some(end) = self.end && self.current >= end { return None; }
//         let res = self.current;
//         self.current = self.current;

//         Some(res)
//     }
// }

#[derive(Debug, Clone)]
pub(crate) struct Tile {
    id: TileId,
    raw_texture: ColorImage,

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

    fn empty(id: TileId, size: [usize; 2]) -> Self {
        Self {
            id,
            raw_texture: utils::empty_image(size)
        }
    }
}

pub(crate) struct TilesContainer {
    pub(crate) inner: IndexedBy<TileId, Tile>
}

impl TilesContainer {
    pub fn new(length: usize, size: [usize; 2]) -> Self {
        let inner = unsafe {
            IndexedBy::new(Vec::from_fn(length, |id| Tile::empty(TileId::from(id), size)))
        };
        Self { inner }
    }

    pub fn iter_ids(&self) -> IdIterator<TileId> {
        self.inner.iter_ids()
    }

    pub fn iter(&self) -> core::slice::Iter<'_, Tile> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, Tile> {
        self.inner.iter_mut()
    }
}

impl Index<TileId> for TilesContainer {
    type Output = Tile;
    fn index(&self, index: TileId) -> &Self::Output {
        &self.inner[index]
    }
}

// pub struct IndexedByTileId<T> {
//     inner: Vec<T>
// }

// impl<T> IndexedByTileId<T> {
//     pub fn new(inner: Vec<T>) -> Self {
//         Self { inner }
//     }

//     pub fn with_repeated(value: T, size: usize) -> Self 
//     where
//         T :Clone
//     {
//         Self { inner: vec![value; size] }
//     }

//     pub fn iter_ids(&self) -> Box<dyn Iterator<Item = TileId>> {
//         Box::new(
//             (0..self.inner.len()).map(|item| TileId::from(item))
//         )
//     }

//     pub fn iter(&self) -> core::slice::Iter<'_, T> {
//         self.inner.iter()
//     }

//     pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
//         self.inner.iter_mut()
//     }
// }

// impl<T> Index<TileId> for IndexedByTileId<T> {
//     type Output = T;
//     fn index(&self, index: TileId) -> &Self::Output {
//         &self.inner[index.0 as usize]
//     }
// }

// impl<T> IndexMut<TileId> for IndexedByTileId<T> {
//     fn index_mut(&mut self, index: TileId) -> &mut Self::Output {
//         &mut self.inner[index.0 as usize]
//     }
// }

// impl<T> std::ops::Deref for IndexedByTileId<T> {
//     type Target = Vec<T>;
//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

// impl<T> std::ops::DerefMut for IndexedByTileId<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner
//     }
// }