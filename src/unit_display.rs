use crate::{game, map, Texture};

#[derive(getset::CloneGetters)]
#[getset(get_clone = "pub")]
pub struct UnitDisplay {
    tile_id: game::TileId,
    
    texture: Texture,
}

impl UnitDisplay {
    pub fn new(tile_id: game::TileId, texture: Texture) -> Self {
        Self { tile_id, texture }
    }
}
