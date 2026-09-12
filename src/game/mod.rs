use std::{sync::{Arc, Mutex, atomic::Ordering, mpsc::{self, TryRecvError}}};

use egui::{Color32, ColorImage};

use crate::{consts, map::IdsMap, tile};

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


