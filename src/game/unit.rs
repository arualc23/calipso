use std::{ops::Deref, sync::mpsc::{Receiver, Sender}};

use egui::TextureHandle;

use crate::{CLOSING_REQUESTED, game::player, id::{IdIterator, IndexedBy}, id_derives, tile::{self, TileId}, utils};


id_derives!{pub UnitId}


pub struct Unit {
    id: UnitId,
    tile: tile::TileId,
    controller: player::PlayerId,
    exists: bool,
    texture: TextureHandle,
}

impl Unit {
    pub fn new(id: UnitId, tile: tile::TileId, controller: player::PlayerId, exists: bool, texture: TextureHandle) -> Self {
        Self { id, tile, controller, exists, texture }
    }

    fn get_dislay(&self) -> UnitDisplay {
        UnitDisplay { tile_id: self.tile, unit_id: self.id, texture: self.texture.clone() }
    }
}
struct UnitStorageUninit {
    inner: IndexedBy<UnitId, Unit>,
    tiles_count: usize,

}

impl UnitStorageUninit {
    fn try_new(mut units: Vec<Unit>, tiles_count: usize) -> Option<Self> {
        units.sort_by(|a, b| a.id.cmp(&b.id));
        
        let ids: Vec<_> = units.iter().map(|unit| unit.id).collect();
        
        let last = match ids.last() {
            Some(v) => v,
            None => return Some(Self {
                inner: IndexedBy::empty(),
                tiles_count
            })
        };

        //If in a sorted list (of positive elements) the last element is the list's length - 1 and there are no duplicates, then each element must be its own index.
        if ids.len() != <UnitId as Into<usize>>::into(*last) + 1 || utils::has_duplicates(&ids) { return None; }

        //Safety: previous checks ensure that the ids are correct.
        unsafe {Some(Self {
            inner: IndexedBy::new(units),
            tiles_count
        })}
    }

    fn current_positions(&self) -> CurrentPositions {
        let mut res = IndexedBy::filled_with(self.tiles_count, || None);
        self.inner.iter().filter(|unit| unit.exists).for_each(|unit| res[unit.tile] = Some(unit.get_dislay()));
        res
    }

    fn create_units_display(&self) -> (AllUnitsDisplay, Sender<CurrentPositions>) {
        // log::info!("create_units_display called");
        let (send, recv) = std::sync::mpsc::channel();
        // log::info!("channel obtained");
        let mut units_display = AllUnitsDisplay {inner: vec![], recv};
        // log::info!("AllUniDisp obtained");
        units_display.update_inner(self.current_positions());
        // log::info!("AllUniDisp updated");
        (units_display, send)
        
    }
}

pub struct UnitStorage {
    inner: IndexedBy<UnitId, Unit>,
    tiles_count: usize,
    send: Sender<CurrentPositions>
}

impl UnitStorage {

    pub fn try_new(units: Vec<Unit>, tiles_count: usize) -> Option<(Self, AllUnitsDisplay)> {
        let uninit = UnitStorageUninit::try_new(units, tiles_count)?;
        let (unit_display, send) = uninit.create_units_display();
        let UnitStorageUninit { inner, tiles_count } = uninit;
        Some((Self {
            inner,
            tiles_count,
            send
        }, unit_display))
    }

    pub fn current_positions(&self) -> CurrentPositions {
        let mut res = IndexedBy::filled_with(self.tiles_count, || None);
        self.inner.iter().filter(|unit| unit.exists).for_each(|unit| res[unit.tile] = Some(unit.get_dislay()));
        res
    }

    pub fn update(&self) -> Result<(), std::sync::mpsc::SendError<CurrentPositions>> {
        self.send.send(self.current_positions())
    }
}

impl Deref for UnitStorage {
    type Target = IndexedBy<UnitId, Unit>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub type CurrentPositions = IndexedBy<tile::TileId, Option<UnitDisplay>>;

#[derive(getset::CloneGetters)]
#[getset(get_clone = "pub")]
pub struct UnitDisplay {
    tile_id: tile::TileId,
    unit_id: UnitId,
    texture: TextureHandle,
}

impl UnitDisplay {
    pub fn new(tile_id: tile::TileId, texture: TextureHandle, unit_id: UnitId) -> Self {
        Self { tile_id, texture, unit_id }
    }
}

pub struct AllUnitsDisplay {
    inner: Vec<UnitDisplay>,
    recv: Receiver<CurrentPositions>,
}

impl AllUnitsDisplay {
    fn update_inner(&mut self, data: CurrentPositions) {
        self.inner = data.into_inner().into_iter().flatten().collect();
    }

    pub fn update(&mut self) {
        match self.recv.try_recv() {
            Ok(val) => {
                self.update_inner(val);
            },
            Err(e) => {match e {
                std::sync::mpsc::TryRecvError::Empty => (),
                std::sync::mpsc::TryRecvError::Disconnected => {
                    CLOSING_REQUESTED.store(true, std::sync::atomic::Ordering::Release);
                    log::error!("Unit update channel input disconnected. Closing requested...")
                }
            }},
        }
    }

    pub fn as_slice(&self) -> &[UnitDisplay] {
        &self.inner
    }
}