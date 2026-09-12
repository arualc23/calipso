// pub type Texture = std::sync::Arc<egui::TextureHandle>;
pub const UV: egui::Rect = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
pub static BACKGROUND_LAYER: std::sync::LazyLock<egui::LayerId> = std::sync::LazyLock::new(|| egui::LayerId::background());
pub(crate) const PROCESS_COUNT: usize = 16;