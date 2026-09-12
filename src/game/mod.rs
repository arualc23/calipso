use std::{ops::{Add, Index, IndexMut}, sync::{Arc, Mutex, atomic::Ordering, mpsc::{self, TryRecvError}}};

use egui::{Color32, ColorImage};

use crate::{consts, map::IdsMap, utils::{self, color_image_to_iter}};

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
        let inner: u32 = ((value.r() as u32) << 24 ) |
                         ((value.g() as u32) << 16 ) |
                         ((value.b() as u32) << 8  ) |
                         ((value.a() as u32));
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

#[derive(Debug, Clone)]
pub(crate) struct Tile {
    id: TileId,
    raw_texture: ColorImage,

}

impl Tile {
    fn paste_onto_canvas(&self, canvas: &mut ColorImage) {

        let iter = utils::color_image_to_iter(&self.raw_texture).enumerated();
        for (pos, &color) in iter {
            // if color == Color32::TRANSPARENT && canvas[pos] != color { log::info!("what"); }
            if color == Color32::TRANSPARENT { continue; }
            canvas[pos] = color;
        }
    }

    fn paste_from_image(&mut self, image: impl AsRef<ColorImage>, ids_map: impl AsRef<IdsMap>) {
        for (pos, pixel) in color_image_to_iter(image.as_ref()).enumerated().filter(|(pos, _)| self.id == ids_map.as_ref()[*pos].into()) {
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

impl<T> std::ops::Deref for IndexedByTileId<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for IndexedByTileId<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct LogicalMap {
    map: TilesContainer,
    real_image: ColorImage,
    updated: bool,
}

impl LogicalMap {
    pub fn get_real_image(&self) -> &ColorImage {
        &self.real_image
    }
    fn update_tile_texture(&mut self, tile_id: TileId) {
        self.map[tile_id].paste_onto_canvas(&mut self.real_image);
    }

    pub(crate) fn new(real_image: Arc<ColorImage>, ids_map: Arc<IdsMap>) -> Self {

        let length = ids_map.max().into();
        let mut map = TilesContainer::new(length, real_image.size);
        let chunks = map.inner.chunks_mut(length.div_ceil(consts::PROCESS_COUNT));

        std::thread::scope(|s| {
            let mut processes = [const {None}; consts::PROCESS_COUNT];
            for (i, chunk) in chunks.enumerate() {
                let real_image = real_image.clone();
                let ids_map = ids_map.clone();
                processes[i] = Some(s.spawn(move || {
                    for tile in chunk {
                        tile.paste_from_image(real_image.clone(), ids_map.clone());
                    }
                }));
            }

            processes.into_iter().for_each(|option| {option.and_then(|handle| Some(handle.join()));});
        });

        

        

        Self { map, real_image: (*real_image).clone(), updated: false }
    }
}

// struct UnsafePointer<T> (*const T);

// impl<T> Deref for UnsafePointer<T> {
//     type Target = *const T;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// unsafe impl<T> Send for UnsafePointer<T> {}

// struct UnsafeImagePointer {
//     inner: *mut ColorImage
// }

// impl Clone for UnsafeImagePointer {
//     fn clone(&self) -> Self {
//         Self {inner: self.inner}
//     }
// }

// impl UnsafeImagePointer {
//     fn get(&mut self) -> &mut ColorImage {
//         unsafe {self.inner.as_mut_unchecked()}
//     }
// }

// unsafe impl Send for UnsafeImagePointer {}
// unsafe impl Sync for UnsafeImagePointer {}

// struct PointerIterator<T>(std::ops::Range<*const T>);
// impl<T: Clone> Iterator for PointerIterator<T> {
//     type Item = *const T;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.0.is_empty() { return None; }
//         let res = self.0.start;
//         unsafe { self.0.start = self.0.start.add(1); }
//         Some(res)
//     }
// }


// fn paste_all_onto_canvas(map: &TilesContainer, canvas: &mut ColorImage) {
//     const PROCESS_COUNT: usize = 16;
//     // let cell = UnsafeCell::new(canvas);
//     let image = UnsafeImagePointer { inner: canvas as *mut ColorImage };
//     let max = map.inner.inner.len();
//     let chunks = map.inner.chunks(max.div_ceil(PROCESS_COUNT) );
//     let mut processes = [const {None}; PROCESS_COUNT];
//     for (i, chunk) in chunks.enumerate() {
//         let chunk = chunk.as_ptr_range();
//         let iter: Vec<_> = PointerIterator(chunk).map(|item| UnsafePointer(item)).collect();
//         let image = image.clone();
//         processes[i] = Some(std::thread::spawn(move || {
//             let mut tmp = image;
//             let image = tmp.get();
//             for tile in iter {
//                 unsafe {(*tile.0).paste_onto_canvas(image)};
//             }
//         }));
//     }

//     for mut process in processes {let _ = process.take().and_then(|handle| Some(handle.join().unwrap())); }
// }

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


