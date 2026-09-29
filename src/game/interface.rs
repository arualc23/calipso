use std::{fmt::Debug};

use enum_map::{Enum, EnumMap};
use proc_macros_crate::ArrayVariants;
use serde::{Serialize, de::DeserializeOwned};

use crate::{game::LogicalMap, utils};


pub fn send<FromGUI: Clone + Debug>(ctx: egui::Context, msg: FromGUI, map_opt: Option<&crate::map::Map>, sender: &mut utils::DBufferWriter<(InputSnapshot, FromGUI)>) {
    let input_snapshot = ctx.input(|i| {
        
        InputSnapshot {
            pointer_state: i.pointer.clone(),
            over_tile: map_opt.and_then(|map| map.get_tile_id_from_cursor(i.pointer.latest_pos()?)),
            keys_down: i.keys_down.clone(),
        }
    });

    sender.write((input_snapshot, msg));
}

pub fn init_game_loop<FromGUI, ToGUI, ClientState, ServerState, ToServer, FromServer>(
    client_loop_body: impl FnMut(FromServer, &InputReader, FromGUI, &mut ClientState, &mut LogicalMap) -> (ToGUI, ToServer) + Send + 'static, 
    server_loop_body: impl super::server::ServerLoopBody<ServerState, FromServer, ToServer,>,
    logical_map: crate::game::LogicalMap, 
    real_image: std::sync::Arc<std::sync::Mutex<egui::ColorImage>>,
    client_state: ClientState,
    server_state: ServerState
) -> (utils::DBufferWriter<(InputSnapshot, FromGUI)>, utils::DBufferReader<ToGUI>)
where
    FromGUI: Default + Clone + Send + 'static,
    ToGUI: Clone + Send + 'static + Default + Debug,
    ClientState: Send + 'static,
    ServerState: Send + 'static,
    FromServer: Serialize + DeserializeOwned,
    ToServer: Serialize + DeserializeOwned + Default,

{
    let (to_gui, from_gui) = utils::dbuffer::new((InputSnapshot::default(), FromGUI::default()));
    let (updates_send, updates_recv) = utils::dbuffer::new(ToGUI::default());
    super::server::init_server_loop(server_loop_body, server_state, super::server::SERVER_SOCKET_ADDRESS, logical_map.map.clone());
    let _ = std::thread::spawn(move || crate::game::game_loop(
        client_loop_body, 
        logical_map, 
        real_image, 
        from_gui, 
        client_state, 
        super::server::SERVER_SOCKET_ADDRESS,
        updates_send
    ));

    (to_gui, updates_recv)
}

#[derive(Debug, Default, Clone)]
pub struct InputSnapshot {
    pub pointer_state: egui::PointerState,
    pub over_tile: Option<crate::tile::TileId>,
    pub keys_down: std::collections::HashSet<egui::Key>,
}

#[derive(Debug, Default)]
pub struct InputReader {
    pub pointer_state: egui::PointerState,
    pub over_tile: Option<crate::tile::TileId>,
    pub keys_down: std::collections::HashSet<egui::Key>,

    pub click_handler: ClickHandler,
}

impl InputReader {
    pub fn button_clicked(&self, button: PointerButton) -> bool {
        self.click_handler.check(button, &self.pointer_state)
    }

    pub(crate) fn update_snapshot(&mut self, input: InputSnapshot) {
        let InputSnapshot { pointer_state, over_tile, keys_down } = input;
        
        self.pointer_state = pointer_state;
        self.over_tile = over_tile;
        self.keys_down = keys_down;
    }

    pub(crate) fn update_click_handler(&mut self) {
        self.click_handler.update_previous(&self.pointer_state);
    }
}

#[derive(Debug, Default)]
pub struct ClickHandler {
    previous: EnumMap<PointerButton, bool>,
}

impl ClickHandler {
    pub(crate) fn update_previous(&mut self, input: &egui::PointerState) {
        for button in PointerButton::VARIANTS {
            self.previous[button] = input.button_down(button.into());
        }
    }

    fn check(&self, button: PointerButton, input: &egui::PointerState) -> bool {
        (!self.previous[button]) && input.button_down(button.into())
    }
}

#[derive(Debug, Enum, Clone, Copy, ArrayVariants)]
pub enum PointerButton {
    Primary, 
    Secondary,
    Middle,
    Extra1,
    Extra2
}

impl Into<egui::PointerButton> for PointerButton {
    fn into(self) -> egui::PointerButton {
        match self {
            Self::Extra1 => egui::PointerButton::Extra1,
            Self::Extra2 => egui::PointerButton::Extra2,
            Self::Primary => egui::PointerButton::Primary,
            Self::Secondary => egui::PointerButton::Secondary,
            Self::Middle => egui::PointerButton::Middle,
        }
    }
}

pub type FromGUIChannel<Data> = (InputSnapshot, Data);