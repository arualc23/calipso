// use std::any::Any;

use std::sync::mpsc::{Receiver, Sender};

use serde::{Serialize, de::DeserializeOwned};

use crate::{game::LogicalMap, utils};

// pub trait InputSender {
//     type InputMsg;
//     fn ctx(&self) -> egui::Context;
//     fn get_sender(&self) -> &std::sync::mpsc::Sender<FullMessage<Self::InputMsg>>;


//     fn send(&self, msg: Self::InputMsg, map_opt: Option<&crate::map::Map>) {
//         let input_snapshot = self.ctx().input(|i| {
            
//             InputSnapshot {
//                 pointer_state: i.pointer.clone(),
//                 over_tile: map_opt.and_then(|map| map.get_tile_id_from_cursor(i.pointer.latest_pos()?)),
//                 keys_down: i.keys_down.clone(),
//             }
//         });


//         let _ = self.get_sender().send((input_snapshot, msg));
        
//     }
// }

pub fn send<InputMsg>(ctx: egui::Context, msg: InputMsg, map_opt: Option<&crate::map::Map>, sender: &Sender<FullMessage<InputMsg>>) {
    let input_snapshot = ctx.input(|i| {
        
        InputSnapshot {
            pointer_state: i.pointer.clone(),
            over_tile: map_opt.and_then(|map| map.get_tile_id_from_cursor(i.pointer.latest_pos()?)),
            keys_down: i.keys_down.clone(),
        }
    });


    let _ = sender.send((input_snapshot, msg));
    
}

pub fn init_game_loop<InputMsg, OutputMsg, ClientState, ServerState, ToServer, FromServer>(
    client_loop_body: impl GameLoop<InputMsg, OutputMsg, ClientState, ToServer, FromServer>, 
    server_loop_body: impl super::server::ServerLoopBody<ServerState, FromServer, ToServer,>,
    logical_map: crate::game::LogicalMap, 
    real_image: std::sync::Arc<std::sync::Mutex<egui::ColorImage>>,
    // units: unit::UnitStorage,
    client_state: ClientState,
    server_state: ServerState
) -> (std::sync::mpsc::Sender<(InputSnapshot, InputMsg)>, utils::DBufferReader<OutputMsg>)
where
    InputMsg: Default + Send + 'static,
    OutputMsg: Clone + Send + 'static + Default,
    ClientState: Send + 'static,
    ServerState: Send + 'static,
    FromServer: Serialize + DeserializeOwned,
    ToServer: Serialize + DeserializeOwned + Default,

{

    let (send, recv) = std::sync::mpsc::channel();
    // let dbuffer = std::pin::Pin
    // let (updates_send, updates_recv) = std::sync::mpsc::channel();
    let (updates_send, updates_recv) = utils::dbuffer::new(OutputMsg::default());
    super::server::init_server_loop(server_loop_body, server_state, super::server::SERVER_SOCKET_ADDRESS); //TODO: replace () with FromServer 
    let _ = std::thread::spawn(move || crate::game::game_loop(
        client_loop_body, 
        logical_map, 
        real_image, 
        recv, 
        client_state, 
        super::server::SERVER_SOCKET_ADDRESS,
        // dbg_send
        updates_send
    ));

    (send, updates_recv)
}


pub type FullMessage<InputMsg> = (InputSnapshot, InputMsg);
pub trait GameLoop<InputMsg, OutputMsg, GameState, ToServer, FromServer> = FnMut(FromServer, &FullMessage<InputMsg>, &mut GameState, &mut LogicalMap) -> (OutputMsg, ToServer) + Send + 'static;

#[derive(Debug, Default)]
pub struct InputSnapshot {
    pub pointer_state: egui::PointerState,
    pub over_tile: Option<crate::tile::TileId>,
    pub keys_down: std::collections::HashSet<egui::Key>
}