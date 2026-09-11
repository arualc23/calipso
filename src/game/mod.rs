use std::{collections::HashSet, ops::{Add, Index, IndexMut}, sync::{Arc, Mutex, atomic::Ordering, mpsc::{self, Sender, TryRecvError}}};

use egui::{Color32, ColorImage, Context, Key, PointerState};

use crate::utils;

pub mod interface;
use interface::{FullMessage, GameLoop, InputSnapshot};

pub const NULL: Color32 = Color32::from_rgba_premultiplied(0, 0 ,0, 0);

pub struct PlayerId;
pub struct Player {
    id: PlayerId
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct TileId {
    inner: u32
} 

impl std::fmt::Display for TileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
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
        // log::info!("{:?}", value);
        let inner: u32 = ((value.r() as u32) << 24 ) |
                         ((value.g() as u32) << 16 ) |
                         ((value.b() as u32) << 8  ) |
                         ((value.a() as u32));
        // let inner = unsafe {(&raw const value).cast::<u32>().read() };
        Self { inner }
    }
}

impl Into<Color32> for TileId {
    fn into(self) -> Color32 {
        Color32::from_rgba_premultiplied(
            (self.inner >> 24) as u8,
            (self.inner >> 16) as u8,
            (self.inner >>  8) as u8,
            (self.inner >>  0) as u8,
        )
    }
}

impl Into<usize> for TileId {
    fn into(self) -> usize {
        self.inner as usize
    }
}

impl From<usize> for TileId {
    fn from(value: usize) -> Self {
        Self { inner: value as u32 }
    }
}

impl Add<u32> for TileId {
    type Output = TileId;
    fn add(self, rhs: u32) -> Self::Output {
        Self {inner: self.inner + rhs}
    }
}

///end excluded, start included
pub struct TileIdIter {
    current: TileId,
    end: Option<TileId>,
}

impl TileIdIter {
    pub fn new(start: impl Into<TileId>, end: impl Into<Option<TileId>>) -> Self {
        Self {
            current: start.into(),
            end: end.into()
        }
    }
}

impl Iterator for TileIdIter {
    type Item = TileId;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(end) = self.end && self.current >= end { return None; }
        let res = self.current;
        self.current = self.current + 1;

        Some(res)
    }
}


// pub struct TileIdRange {
//     start: TileId,
//     end: TileId,
//     start_exclusive: bool,
//     end_exclusive: bool,
// }

// impl TileIdRange {
//     ///Returns None if end >= start.
//     pub fn new(start: impl Into<TileId>, end: impl Into<TileId>, start_exclusive: bool, end_exclusive: bool) -> Option<Self> {
//         let (start, end) = (start.into(), end.into());
//         if start < end {
//             Some(Self { start, end, start_exclusive, end_exclusive })
//         } else { None }
//     }
// }

// impl std::ops::RangeBounds<TileId> for TileIdRange {
//     fn start_bound(&self) -> std::ops::Bound<&TileId> {
//         if self.start_exclusive {std::ops::Bound::Excluded(&self.start)}
//         else {std::ops::Bound::Included(&self.start)}
//     }

//     fn end_bound(&self) -> std::ops::Bound<&TileId> {
//         if self.end_exclusive {std::ops::Bound::Excluded(&self.end)}
//         else {std::ops::Bound::Included(&self.end)}
//     }
// }

pub(crate) struct Tile {
    id: TileId,
    raw_texture: ColorImage,

}

impl Tile {
    fn paste_onto_canvas(&self, canvas: &mut ColorImage) {

        let iter = utils::color_image_to_iter(&self.raw_texture).enumerated();
        for (pos, &color) in iter {
            canvas[pos] = color;
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
    inner: IndexedByTileId<Tile>
}

impl TilesContainer {
    pub fn new(length: usize, size: [usize; 2]) -> Self {
        let inner = IndexedByTileId::new(Vec::from_fn(length, |id| Tile::empty(TileId::from(id), size)));
        Self { inner }
    }

    pub fn iter_ids(&self) -> Box<dyn Iterator<Item = TileId>> {
        self.inner.iter_ids()
    }

    pub fn iter(&self) -> core::slice::Iter<'_, Tile> {
        self.inner.iter()
    }
}

impl Index<TileId> for TilesContainer {
    type Output = Tile;
    fn index(&self, index: TileId) -> &Self::Output {
        &self.inner[index]
    }
}

pub struct IndexedByTileId<T> {
    inner: Vec<T>
}

impl<T> IndexedByTileId<T> {
    pub fn new(inner: Vec<T>) -> Self {
        Self { inner }
    }

    pub fn with_repeated(value: T, size: usize) -> Self 
    where
        T :Clone
    {
        Self { inner: vec![value; size] }
    }

    pub fn iter_ids(&self) -> Box<dyn Iterator<Item = TileId>> {
        Box::new(
            (0..self.inner.len()).map(|item| TileId::from(item))
        )
    }

    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.inner.iter_mut()
    }
}

impl<T> Index<TileId> for IndexedByTileId<T> {
    type Output = T;
    fn index(&self, index: TileId) -> &Self::Output {
        &self.inner[index.inner as usize]
    }
}

impl<T> IndexMut<TileId> for IndexedByTileId<T> {
    fn index_mut(&mut self, index: TileId) -> &mut Self::Output {
        &mut self.inner[index.inner as usize]
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
            log::info!("iteration, tile {:?}", tile.id);
            tile.paste_onto_canvas(&mut real_image);
        }

        Self {
            map,
            real_image,
            updated: true,
        }
    }
}



pub(crate) fn game_loop<Msg>(
    mut body: impl GameLoop<Msg>, 
    mut logical_map: LogicalMap, 
    real_image: Arc<Mutex<ColorImage>>, 
    input_channel: mpsc::Receiver<FullMessage<Msg>>,
) 
where
    Msg: Default
{
    let mut input = <(InputSnapshot, Msg)>::default();
    loop {
        if crate::CLOSING_REQUESTED.load(Ordering::Acquire) {
            log::info!("Close requested! Exiting...");
            break;
        }

        input = match input_channel.try_recv() {
            Ok(val) => val,
            Err(e) => {
                if let TryRecvError::Disconnected = e {
                    crate::CLOSING_REQUESTED.store(true, Ordering::Release);
                    log::error!("Main thread disconnected! This is the final iteration."); 
                }
                input
            }
        };
        body(&input);

        if logical_map.updated {
            let mut image = real_image.lock().unwrap();
            *image = logical_map.real_image.clone();
            logical_map.updated = false;
        }
    }
}


