use egui::TextureHandle;

use crate::{game, map};

#[derive(getset::CloneGetters)]
#[getset(get_clone = "pub")]
pub struct UnitDisplay {
    tile_id: game::TileId,
    
    texture: TextureHandle,
}

impl UnitDisplay {
    pub fn new(tile_id: game::TileId, texture: TextureHandle) -> Self {
        Self { tile_id, texture }
    }
}
