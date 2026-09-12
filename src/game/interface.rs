use crate::game::{LogicalMap, unit};

pub trait InputSender {
    type Msg;
    fn ctx(&self) -> egui::Context;
    fn get_sender(&self) -> &std::sync::mpsc::Sender<FullMessage<Self::Msg>>;


    fn send(&self, msg: Self::Msg, map_opt: Option<&crate::map::Map>) {
        let input_snapshot = self.ctx().input(|i| {
            
            InputSnapshot {
                pointer_state: i.pointer.clone(),
                over_tile: map_opt.and_then(|map| map.get_tile_id_from_cursor(i.pointer.latest_pos()?)),
                keys_down: i.keys_down.clone(),
            }
        });


        let _ = self.get_sender().send((input_snapshot, msg));
        
    }
}

pub fn init_game_loop<Msg, GameState>(
    game_loop_body: impl GameLoop<Msg, GameState>, 
    logical_map: crate::game::LogicalMap, 
    real_image: std::sync::Arc<std::sync::Mutex<egui::ColorImage>>,
    // units: unit::UnitStorage,
    state: GameState
) -> std::sync::mpsc::Sender<(InputSnapshot, Msg)> 
where
    Msg: Default + Send + 'static,
    GameState: Send + 'static
{
    let (send, recv) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || crate::game::game_loop(game_loop_body, logical_map, real_image, recv, state, ));

    send
}


pub type FullMessage<Msg> = (InputSnapshot, Msg);
pub trait GameLoop<Msg, GameState> = FnMut(&FullMessage<Msg>, &mut GameState, &mut LogicalMap) + Send + 'static;

#[derive(Debug, Default)]
pub struct InputSnapshot {
    pub pointer_state: egui::PointerState,
    pub over_tile: Option<crate::tile::TileId>,
    pub keys_down: std::collections::HashSet<egui::Key>
}