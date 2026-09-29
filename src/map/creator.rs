use egui::{Color32, ColorImage, Pos2, Rect, pos2, vec2};
use std::sync::{Arc, Mutex};

use crate::{id::{IdIterator, IndexedBy}, map::IdsMap, tile::TileId, utils::color_image_to_iter};

pub type Bboxes = IndexedBy<TileId, Rect>;

pub fn compute_bbox(map: &IdsMap) -> Bboxes {
    log::info!("compute bbox called");
    let res = Arc::new(Mutex::new(IndexedBy::filled(<TileId as Into<usize>>::into(map.max()) + 1, Rect::ZERO,)));
    let length: usize = map.max().into();

    let tmp = IdIterator::new(0.into(), map.max()).collect::<Vec<_>>();

    crate::THREAD_POOL.lock().unwrap().scope(|s| {
        let chunks = tmp.chunks(length.div_ceil(*crate::PROCESS_COUNT));

        for (_, chunk) in chunks.enumerate() {
            let thread_side = res.clone();
            s.add_job(Box::new(move || {
                for &id in chunk {
                    let color = id.into();
                    let center = find_center(color, &*map);
                    let radius = binary_search_biggest_rect(color, center, &*map) as f32 * 2.0;
                    thread_side.lock().unwrap()[id] = Rect::from_center_size(center, vec2(radius, radius));
                }

            }));
        }
    });  
    
    std::mem::replace(Arc::into_inner(res).unwrap().get_mut().unwrap(), IndexedBy::empty())
}

fn find_center(color: Color32, image: &ColorImage) -> Pos2 {

    
    let (sumx, sumy, count) = color_image_to_iter(&image).enumerated()
        .filter(|(_, col)| **col == color)
        .map(|(index, _)| index)
        .fold((0, 0, 0.0), |acc, (x, y)| (acc.0 + x, acc.1 + y, acc.2 + 1.0));
    pos2(sumx as f32/count , sumy as f32/count)

}

fn binary_search_biggest_rect(color: Color32, center: Pos2, image: &ColorImage) -> usize {
    const RECT_MAX_SIZE: usize = 1000;
    let dist = center.x.min(RECT_MAX_SIZE as f32).min(center.y).min(image.size[0] as f32 - center.x).min(image.size[1] as f32 - center.y);
    let vector: Vec<usize> = (1..=dist as usize).collect();

    match vector.binary_search_by(|&r| {
        let rect = Rect::from_center_size(center, vec2(r as f32, r as f32));
        let image_part = image.region(&rect, None);
        if color_image_to_iter(&image_part).any(|pixel| *pixel != color) {std::cmp::Ordering::Greater}
        else {std::cmp::Ordering::Less}
    }) {
        Ok(v) | Err(v) => v
    }

}