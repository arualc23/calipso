use std::{fmt::Debug, ops::Index, sync::Arc};
use egui::{Color32, ColorImage, Context, Painter, Pos2, Rect, TextureHandle, Ui, Vec2, pos2};
use crate::{map, utils};

const SCROLL_SCALE: f32 = 500.0;

pub struct Map {
    pub texture: Arc<TextureHandle>,
    pub rect: Rect,
    pub raw: ColorImage,
    pub starting_rect: Rect,
    pub starting_diag: f32,
}

impl Map {
    pub fn update(&mut self, camera_coords: Vec2, cursor_coords: Vec2, scroll: f32) {
        Self::update_point(&mut self.rect.min, cursor_coords, scroll, camera_coords);
        Self::update_point(&mut self.rect.max, cursor_coords, scroll, camera_coords);

    }

    fn update_point(point: &mut Pos2, cursor_coords: Vec2, scroll: f32, camera_coords: Vec2) {
        *point += (*point - camera_coords - cursor_coords).to_vec2() * scroll/SCROLL_SCALE;
    }

    pub fn handle_click(&self, cursor_coords: Pos2) -> Color32 {
        let translated_pos = self.current_to_starting_coords(cursor_coords);
        self.get_color(translated_pos.x as isize, translated_pos.y as isize)
    }

    fn get_color(&self, x: isize, y: isize) -> Color32 {
        let (Ok(x), Ok(y)) = (
            usize::try_from(x),
            usize::try_from(y)
        ) else {return Color32::PLACEHOLDER};
        *(self.raw.get((x, y)).unwrap_or(&Color32::PLACEHOLDER))
    }

    fn current_to_starting_coords(&self, pos: Pos2) -> Pos2 {
        self.starting_rect.min + (pos - self.rect.min)/self.rect.size().length()*self.starting_diag
    }

    pub fn new(texture: Arc<TextureHandle>, raw: ColorImage, starting_rect: Rect, ) -> Self {
        let rect = starting_rect;
        let starting_diag = rect.size().length();
        dbg!(raw.size);
        Self {
            texture,
            rect,
            raw,
            starting_rect,
            starting_diag
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

        if ui.input(|i| i.pointer.primary_clicked()) {
            log::debug!("{:?}", self.map().handle_click(cursor_coords));
        }

        self.map().update(camera_coords, cursor_coords.to_vec2(), scroll);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
struct TileId {
    inner: u32
} 

impl From<Color32> for TileId {
    fn from(value: Color32) -> Self {
        let inner: u32 = (value.r() as u32) << 24 +
                         (value.g() as u32) << 16 +
                         (value.b() as u32) << 8;
        Self { inner }
    }
}

#[derive(Debug)]
pub struct LoadMapError(String);

impl LoadMapError {
    fn from_debug(value: impl Debug) -> Self {
        Self(format!("{value:?}"))
    }
}

pub fn load_map(directory_name: &str, ctx: &Context) -> Result<Map, LoadMapError> {
    const RAW_FILE_NAME: &str = "raw.png";
    const VISUAL_FILE_NAME: &str = "vis.png";
    use std::path::Path;
    use std::sync::LazyLock;
    static SAVES_DIR: LazyLock<&Path> = LazyLock::new(|| Path::new("saves"));

    let dir_path = SAVES_DIR.join(directory_name);

    let raw_image = utils::load_image_from_path(dir_path.join(RAW_FILE_NAME))
        .map_err(|e| LoadMapError::from_debug(e))?;
    let map_texture = utils::load_texture_from_path(dir_path.join(VISUAL_FILE_NAME), ctx, directory_name)
        .map_err(|e| LoadMapError::from_debug(e))?;
    let starting_rect = Rect::from_min_max(Pos2::ZERO, pos2(raw_image.size[0] as f32, raw_image.size[1] as f32));
    Ok(Map::new(map_texture, raw_image, starting_rect))

}