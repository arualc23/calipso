use std::{collections::HashSet, fmt::Debug, ops::Index, path::PathBuf, sync::{Arc, Mutex}};
use egui::{Color32, ColorImage, Context, Painter, Pos2, Rect, TextureHandle, Ui, Vec2, pos2};
use log::info;
use crate::{game::{self, TileId}, utils::{self, ASSETS}};

const SCROLL_SCALE: f32 = 500.0;
pub const MAX_MAP_RAW_LEN: usize = 5000 * 5000;

pub struct Map {
    texture: Arc<TextureHandle>,
    rect: Rect,
    ids_map: ColorImage,
    starting_rect: Rect,
    starting_diag: f32,
    raw_image: Arc<Mutex<ColorImage>>,
    ctx: Context
}

impl Map {
    pub fn update(&mut self, camera_coords: Vec2, cursor_coords: Vec2, scroll: f32) {
        Self::update_point(&mut self.rect.min, cursor_coords, scroll, camera_coords);
        Self::update_point(&mut self.rect.max, cursor_coords, scroll, camera_coords);

    }

    pub fn ctx(&self) -> Context {
        self.ctx.clone()
    }

    fn update_point(point: &mut Pos2, cursor_coords: Vec2, scroll: f32, camera_coords: Vec2) {
        *point += (*point - camera_coords - cursor_coords).to_vec2() * scroll/SCROLL_SCALE;
    }

    pub(crate) fn get_tile_id_from_cursor(&self, cursor_coords: Pos2) -> Option<TileId> {
        let translated_pos = self.current_to_starting_coords(cursor_coords);
        self.get_color(translated_pos.x as isize, translated_pos.y as isize)
    }

    fn get_color(&self, x: isize, y: isize) -> Option<TileId> {
        let (Ok(x), Ok(y)) = (
            usize::try_from(x),
            usize::try_from(y)
        ) else {return None};
        self.ids_map.get((x, y)).map(|&color| TileId::from(color))
    }

    fn current_to_starting_coords(&self, pos: Pos2) -> Pos2 {
        self.starting_rect.min + (pos - self.rect.min)/self.rect.size().length()*self.starting_diag
    }

    fn get_new_texture(raw_image: &ColorImage, ctx: &Context) -> Arc<TextureHandle> {
        utils::load_texture_from_image(raw_image, ctx, "real_map_texture")
    }

    pub fn get_raw_image(&self) -> Arc<Mutex<ColorImage>> {
        self.raw_image.clone()
    }

    fn update_texture(&mut self) {
        let image = self.raw_image.lock().unwrap();
        self.texture = 
            Self::get_new_texture(&image, &self.ctx);
    }

    pub fn new(raw_image: Arc<Mutex<ColorImage>>, ids_map: ColorImage, starting_rect: Rect, ctx: Context) -> Self {
        let rect = starting_rect;
        let starting_diag = rect.size().length();
        let texture = Self::get_new_texture(&raw_image.lock().unwrap(), &ctx);
        dbg!(ids_map.size);
        Self {
            texture,
            rect,
            ids_map,
            starting_rect,
            starting_diag,
            raw_image,
            ctx
        }
    }
}

trait SafeImageIndex<Idx>: Index<Idx> {
    fn get(&self, index: Idx) -> Option<&Color32>;
}
impl SafeImageIndex<(usize, usize)> for ColorImage {
    fn get(&self, index: (usize, usize)) -> Option<&Color32> {
        if index.0 < self.size[0] && index.1 < self.size[1] {
            Some(&self[index])
        } else {
            None
        }
    }
}

///Implements painting the given map and updating it with camera movement on the given painter and ui.
pub trait MapDisplay {
    fn map(&mut self) -> &mut Map;
    ///Paints and updates the map.
    fn paint_map(&mut self, ui: &mut Ui, painter: &Painter) {
        painter.image(self.map().texture.id(), self.map().rect, utils::UV, Color32::WHITE);

        let camera_coords = Vec2::ZERO;
        let cursor_coords = ui.input(|i| i.pointer.latest_pos().unwrap_or_default());
        let scroll = ui.input(|i| i.smooth_scroll_delta().y);


        self.map().update(camera_coords, cursor_coords.to_vec2(), scroll);
    }
}

#[derive(Debug)]
pub struct LoadMapError(String);

impl LoadMapError {
    fn from_debug(value: impl Debug) -> Self {
        Self(format!("{value:?}"))
    }
}

pub fn load_map(directory_name: &str, ctx: Context) -> Result<(Map, game::LogicalMap), LoadMapError> {
    const IDS_MAP_FILE_NAME: &str = "raw.png";
    const VISUAL_FILE_NAME: &str = "vis.png";
    use std::sync::LazyLock;
    static MAPS_DIR: LazyLock<PathBuf> = LazyLock::new(|| ASSETS.join("maps"));

    let dir_path = MAPS_DIR.join(directory_name);

    let ids_map = utils::load_image_from_path(dir_path.join(IDS_MAP_FILE_NAME))
        .map_err(|e| LoadMapError::from_debug(e))?;
    let real_map_image = utils::load_image_from_path(dir_path.join(VISUAL_FILE_NAME))
        .map_err(|e| LoadMapError::from_debug(e))?;

    assert_eq!(ids_map.size, real_map_image.size);
    let size = ids_map.size;

    let arc = Arc::new(Mutex::new(real_map_image));
    
    let length = ids_map.as_raw().chunks_exact(4).map(|chunk| u32::from_be_bytes(chunk.try_into().unwrap()) >> 8).max().expect("If there isn't a max there must've been no tiles") as usize;

    let tiles_container = game::TilesContainer::new(length, size);

    let logical_map = game::LogicalMap::new(tiles_container, size);

    let starting_rect = Rect::from_min_max(Pos2::ZERO, pos2(size[0] as f32, size[1] as f32));
    Ok((Map::new(arc, ids_map, starting_rect, ctx), logical_map))

}
