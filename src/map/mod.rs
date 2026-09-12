use std::{collections::HashSet, fmt::Debug, ops::{Deref, DerefMut, Index}, path::PathBuf, sync::{Arc, Mutex}};
use egui::{Color32, ColorImage, Context, Painter, Pos2, Rect, TextureHandle, Ui, Vec2, pos2};
use crate::{BACKGROUND_LAYER, game::{self, unit::{AllUnitsDisplay, UnitDisplay}}, id::IdIterator, map::creator::Bboxes, tile::TileId, utils::{self, ASSETS, color_image_to_iter}};

const SCROLL_SCALE: f32 = 500.0;
pub const MAX_MAP_RAW_LEN: usize = 5000 * 5000;
pub mod creator;

#[derive(Debug, Clone)]
pub struct IdsMap {
    inner: ColorImage,
    max_id: TileId,
}

impl IdsMap {
    pub fn new(inner: ColorImage, province_count_estimate: Option<usize>) -> Option<Self> {
        let mut all_ids = HashSet::with_capacity(province_count_estimate.unwrap_or(10));
        for &pixel in color_image_to_iter(&inner) {
            all_ids.insert(TileId::from(pixel));
        }

        let &max = all_ids.iter().max()?;
        let iter = IdIterator::new(0.into(), max);
        for i in iter {
            if !all_ids.contains(&TileId::from(i)) { return None; }
        }

        Some(Self { inner, max_id: max })
    }

    pub fn max(&self) -> TileId {
        self.max_id
    }
}

impl Deref for IdsMap {
    type Target = ColorImage;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for IdsMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct Map {
    texture: TextureHandle,
    rect: Rect,
    ids_map: IdsMap,
    starting_rect: Rect,
    starting_diag: f32,
    raw_image: Arc<Mutex<ColorImage>>,
    ctx: Context,
    bboxes: Bboxes,
}

impl Map {
    fn update_map_raw(&mut self, camera_coords: Vec2, cursor_coords: Vec2, scroll: f32) {
        Self::update_rect(&mut self.rect, camera_coords, cursor_coords, scroll);
        self.bboxes.iter_mut().for_each(|rect| Self::update_rect(rect, camera_coords, cursor_coords, scroll));
    }

    fn update_rect(rect: &mut Rect, camera_coords: Vec2, cursor_coords: Vec2, scroll: f32) {
        Self::update_point(&mut rect.max, cursor_coords, scroll, camera_coords);
        Self::update_point(&mut rect.min, cursor_coords, scroll, camera_coords);
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

    fn get_new_texture(raw_image: &ColorImage, ctx: &Context, options: utils::TextureOptions) -> TextureHandle {
        utils::load_texture_from_image(raw_image, ctx, "real_map_texture", options)
    }

    pub fn get_raw_image(&self) -> Arc<Mutex<ColorImage>> {
        self.raw_image.clone()
    }

    pub fn ids_map(&self) -> &IdsMap {
        &self.ids_map
    }

    fn update_texture(&mut self) {
        let image = self.raw_image.lock().unwrap();
        self.texture = 
            Self::get_new_texture(&image, &self.ctx, utils::TextureOptions::Exact);
    }

    pub fn new(raw_image: Arc<Mutex<ColorImage>>, ids_map: IdsMap, starting_rect: Rect, ctx: Context, bboxes: Bboxes) -> Self {
        let rect = starting_rect;
        let starting_diag = rect.size().length();
        let texture = Self::get_new_texture(&raw_image.lock().unwrap(), &ctx, utils::TextureOptions::Exact);
        dbg!(ids_map.size);
        Self {
            texture,
            rect,
            ids_map,
            starting_rect,
            starting_diag,
            raw_image,
            ctx,
            bboxes
        }
    }

    pub fn bboxes(&self) -> &Bboxes {
        &self.bboxes
    }

    ///Updates the map with camera movement. Must be done before displaying anything. Prefer [Map::run_frame].
    pub fn update_map(&mut self, ui: &mut Ui) {
        let camera_coords = Vec2::ZERO;
        let cursor_coords = ui.input(|i| i.pointer.latest_pos().unwrap_or_default());
        let scroll = ui.input(|i| i.smooth_scroll_delta().y);


        self.update_map_raw(camera_coords, cursor_coords.to_vec2(), scroll);
    }

    ///Prefer [Map::run_frame].
    pub fn paint_map(&self, painter: &Painter) {
        painter.image(self.texture.id(), self.rect, utils::UV, Color32::WHITE);
    }

    ///Prefer [Map::run_frame].
    pub fn paint_units(&self, painter: &Painter, units: &[UnitDisplay]) {
        for unit in units {
            painter.image(unit.texture().id(), self.bboxes[unit.tile_id()], crate::UV, egui::Color32::WHITE);
        }
    }

    ///Paints everything and updates with camera movement. Set painter to None to get the default (background layer painter).
    pub fn run_frame(&mut self, ui: &mut Ui, painter: Option<&Painter>, units: &mut AllUnitsDisplay) {
        self.update_map(ui);

        units.update();

        let painter = match painter {
            Some(val) => val,
            None => &ui.layer_painter(*BACKGROUND_LAYER)
        };

        self.paint_map(painter);
        self.paint_units(painter, units.as_slice());
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

    let ids_map = IdsMap::new(ids_map, Some(50)).ok_or(LoadMapError("Tile ids not unique".to_string()))?;
    
    let length = 1+ ids_map.as_raw().chunks_exact(4).map(|chunk| u32::from_be_bytes(chunk.try_into().unwrap())).max().expect("If there isn't a max there must've been no tiles") as usize;
    log::info!("Loading map with {length} tiles.");

    let arc_ids_map = Arc::new(ids_map);
    let arc_real_map_image = Arc::new(real_map_image);
    let logical_map = game::LogicalMap::new(arc_real_map_image, arc_ids_map.clone());
    let ids_map = Arc::into_inner(arc_ids_map).expect("All threads must be joined by now");

    let raw_image = Arc::new(Mutex::new(logical_map.get_real_image().clone()));

    let starting_rect = Rect::from_min_max(Pos2::ZERO, pos2(size[0] as f32, size[1] as f32));
    let bboxes = creator::compute_bbox(&ids_map);

    Ok((Map::new(raw_image, ids_map, starting_rect, ctx, bboxes), logical_map))

}
