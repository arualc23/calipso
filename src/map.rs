use std::sync::Arc;
use egui::{TextureHandle, Rect, ColorImage, Color32, Vec2, Pos2};

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

trait SafeImageIndex<Idx>: std::ops::Index<Idx> {
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