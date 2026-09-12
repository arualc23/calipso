use std::{ops::{Add, Index, IndexMut}, sync::{Arc, Mutex, atomic::Ordering, mpsc::{self, TryRecvError}}};

use egui::{Color32, ColorImage};

use crate::{consts, map::IdsMap, utils::{self, color_image_to_iter}, tile};

pub mod interface;
use interface::{FullMessage, GameLoop, InputSnapshot};

pub const NULL: Color32 = Color32::from_rgba_premultiplied(0, 0 ,0, 0);

pub struct PlayerId;
pub struct Player {
    id: PlayerId
}

pub struct LogicalMap {
    map: tile::TilesContainer,
    real_image: ColorImage,
    updated: bool,
}

impl LogicalMap {
    pub fn get_real_image(&self) -> &ColorImage {
        &self.real_image
    }
    fn update_tile_texture(&mut self, tile_id: tile::TileId) {
        self.map[tile_id].paste_onto_canvas(&mut self.real_image);
    }

    pub(crate) fn new(real_image: Arc<ColorImage>, ids_map: Arc<IdsMap>) -> Self {

        let length = ids_map.max().into();
        let mut map = tile::TilesContainer::new(length, real_image.size);
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


