use egui::TextureHandle;

use crate::{game::player, id_derives, tile, utils, id::IndexedBy};


id_derives!{pub UnitId}


pub struct Unit {
    id: UnitId,
    tile: tile::TileId,
    controller: player::PlayerId,
    exists: bool,
}

impl Unit {
    pub fn new(id: UnitId, tile: tile::TileId, controller: player::PlayerId, exists: bool) -> Self {
        Self { id, tile, controller, exists }
    }
}
pub struct UnitStorage {
    inner: IndexedBy<UnitId, Unit>,
    tiles_count: usize,
}

impl UnitStorage {
    pub fn try_new(mut units: Vec<Unit>, tiles_count: usize) -> Option<Self> {
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

    pub fn current_positions(&self) -> CurrentPositions {
        let mut res = IndexedBy::filled(self.tiles_count, None);
        self.inner.iter().filter(|unit| unit.exists).for_each(|unit| res[unit.tile] = Some(unit.id));
        res
    }
}

pub type CurrentPositions = IndexedBy<tile::TileId, Option<UnitId>>;

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

// pub struct AllUnitsDisplay {
//     inner: IndexedBy<tile::TileId, Option<UnitDisplay>>
// }

// impl AllUnitsDisplay {
//     pub fn new(elements: Vec<UnitDisplay>) -> Self {
//         for 
//     }
// }