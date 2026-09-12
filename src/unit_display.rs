use egui::TextureHandle;

use crate::{tile};

#[derive(getset::CloneGetters)]
#[getset(get_clone = "pub")]
pub struct UnitDisplay {
    tile_id: tile::TileId,
    
    texture: TextureHandle,
}

impl UnitDisplay {
    pub fn new(tile_id: tile::TileId, texture: TextureHandle) -> Self {
        Self { tile_id, texture }
    }
}
