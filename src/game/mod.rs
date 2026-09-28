use std::{collections::HashSet, fmt::Debug, sync::{Arc, Mutex, atomic::Ordering}};

use egui::{Color32, ColorImage};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{consts, game::interface::InputReader, id::{IndexedBy}, map::IdsMap, tile, utils::{self, DBufferReader}};

pub mod interface;
pub mod unit;
pub mod player;
pub mod server;

use interface::{InputSnapshot};

pub const NULL: Color32 = Color32::from_rgba_premultiplied(0, 0 ,0, 0);

fn get_neighbours(ids_map: &IdsMap) -> IndexedBy<tile::TileId, Vec<tile::TileId>> {
    //We're iterating through all pixels first horizontally, then vertically, and when they change adding that to neighoburs list. 
    //Diagonal is a very edge case and I can't be bothered.
    const MAX_NEIGHBOURS_ESTIMATE: usize = 15;
    let mut previous = tile::TileId::from(ids_map[(0, 0)]);
    let mut res = IndexedBy::filled(ids_map.tiles_count(), HashSet::with_capacity(MAX_NEIGHBOURS_ESTIMATE));

    let width = ids_map.width();
    for (i, id) in utils::color_image_to_horizontal_iter(ids_map).map(|&color| tile::TileId::from(color)).enumerate() {
        if i % width == 0 { previous = id; continue; }
        if previous != id {
            res[previous].insert(id);
            res[id].insert(previous);
            previous = id;
        }
    }

    let height = ids_map.height();
    for (i, id) in utils::color_image_to_vertical_iter(ids_map).map(|&color| tile::TileId::from(color)).enumerate() {
        if i % height == 0 { previous = id; continue; }
        if previous != id {
            res[previous].insert(id);
            res[id].insert(previous);
            previous = id;
        }
    }

    res.into_iter().map(|set| set.into_iter().collect()).collect()
}

pub struct LogicalMap {
    map: IndexedBy<tile::TileId, tile::Tile>,
    real_image: ColorImage,
    updated: bool,
    neighbours: IndexedBy<tile::TileId, Vec<tile::TileId>>,
    player_color_table: IndexedBy<player::PlayerId, Color32>
}

impl LogicalMap {
    /// Remember to manually update the buffered texture with [Self::update_tile_texture] or [Self::update_all_tile_textures] if it was changed.
    pub fn get_real_image(&self) -> &ColorImage {
        &self.real_image
    }

    pub fn update_tile_texture(&mut self, tile_id: tile::TileId) {
        self.map[tile_id].paste_onto_canvas(&mut self.real_image, |id| self.player_color_table[id]);
    }
    pub fn update_all_tile_textures(&mut self) {
        for tile in self.map.iter() {
            tile.paste_onto_canvas(&mut self.real_image, |id| self.player_color_table[id]);
        }
    }

    pub(crate) fn new(
        real_image: Arc<ColorImage>, 
        ids_map: Arc<IdsMap>, 
        movement_costs: &IndexedBy<tile::TileId, f32>, 
        controllers: &IndexedBy<tile::TileId, player::PlayerId>,
        player_color_table: IndexedBy<player::PlayerId, Color32>
    ) -> Self {

        let length = <tile::TileId as Into<usize>>::into(ids_map.max()) + 1usize;
        // let mut map = tile::TilesContainer::new(length, real_image.size, movement_costs);
        let mut map = unsafe { IndexedBy::new(Vec::from_fn(length, |id| {
            let tile_id = tile::TileId::from(id);
            tile::Tile::empty(tile_id, real_image.size, movement_costs[tile_id], controllers[tile_id])
        })
            
        )};
        let chunks = map.chunks_mut(length.div_ceil(consts::PROCESS_COUNT));

        std::thread::scope(|s| {
            let mut processes = [const {None}; consts::PROCESS_COUNT];
            for (i, chunk) in chunks.enumerate() {
                let real_image = real_image.clone();
                let ids_map = ids_map.clone();
                processes[i] = Some(s.spawn(move || {
                    for tile in chunk {
                        tile.paste_from_image(real_image.clone(), ids_map.clone());
                    }
                }));
            }

            processes.into_iter().for_each(|option| {option.and_then(|handle| Some(handle.join()));});
        });

        let neighbours = get_neighbours(&ids_map);

        Self { map, real_image: (*real_image).clone(), updated: false, neighbours, player_color_table }
    }

    pub fn tiles_count(&self) -> usize {
        self.map.len()
    }

    pub fn neighbours(&self, tile: tile::TileId) -> &[tile::TileId] {
        &self.neighbours[tile]
    }

    pub fn get_tile(&self, id: tile::TileId) -> Option<&tile::Tile> {
        self.map.get(id)
    }

    pub fn get_path(&self, tile_from: tile::TileId, tile_to: tile::TileId) -> Path {
        // log::info!("get_path called");
        let raw = utils::dijkstra(&self, tile_from);
        // log::info!("raw obtained: {:#?}", &raw);
        let mut path = Vec::with_capacity(self.tiles_count());
        path.push(tile_to);
        let mut cursor = raw[tile_to].1;
        while cursor != tile_from {
            path.push(cursor);
            cursor = raw[cursor].1;
        }
        Path { inner: path }
    }

    
}

///Stores a path between two tiles. Accessed with its [core::iter::Iterator] implementation. Does not include the starting tile.
#[derive(Deserialize, Serialize,Debug)]
pub struct Path {
    inner: Vec<tile::TileId>,
}

impl Iterator for Path {
    type Item = tile::TileId;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop()
    }
}

impl Path {
    pub fn finished(&self) -> bool {
        self.inner.is_empty()
    }
}


pub(crate) fn game_loop<FromGUI, ToGUI, GameState, ToServer, FromServer>(
    mut body: impl FnMut(FromServer, &InputReader, FromGUI, &mut GameState, &mut LogicalMap) -> (ToGUI, ToServer) + Send + 'static, 
    mut logical_map: LogicalMap, 
    real_image: Arc<Mutex<ColorImage>>, 
    mut from_gui: DBufferReader<(InputSnapshot, FromGUI)>,
    // mut units: unit::UnitStorage,
    mut state: GameState,
    server_socket: std::net::SocketAddr,
    // dbg_send: Sender<Box<dyn Any + Send>>,
    mut updates_send: utils::DBufferWriter<ToGUI>
) 
where
    FromGUI: Default + Clone,
    ToGUI: Clone + Debug,
    FromServer: DeserializeOwned,
    ToServer: Serialize,
{
    log::info!("Client game loop called");
    // let mut input = <(InputSnapshot, FromGUI)>::default();
    //Server always needs to initialize first
    std::thread::sleep(std::time::Duration::from_millis(1000));
    log::info!("Connecting to server at {server_socket:?}...");
    let mut stream = std::net::TcpStream::connect(server_socket).unwrap();
    let mut input_reader = InputReader::default();
    loop {
        let server_input: FromServer = match server::read_from_stream(&mut stream) {
            Ok(val) => val,
            Err(e) => match e.kind() {
                std::io::ErrorKind::UnexpectedEof => { log::info!("Connection lost! Exiting..."); break; },
                _ => unimplemented!("Unimplemented error type when reading from server stream: {e}")
            }
        };
        if crate::CLOSING_REQUESTED.load(Ordering::Acquire) {
            log::info!("Close requested! Exiting...");
            break;
        }

        // let server_input: FromServer = server::read_from_stream(&mut stream).unwrap();

        let (input_snapshot, from_gui) = from_gui.read();
        // log::info!("Before upadate snapgh: {}", input_snapshot.pointer_state.any_down());
        input_reader.update_snapshot(input_snapshot);

        let (to_gui, to_server) = body(server_input, &input_reader, from_gui, &mut state, &mut logical_map);

        if logical_map.updated {
            let mut image = real_image.lock().unwrap();
            *image = logical_map.real_image.clone();
            logical_map.updated = false;
        }

        updates_send.write(to_gui);
        // updates_send.swap();

        input_reader.update_click_handler();

        // let data = ();

        server::write_to_stream(&mut stream, to_server).unwrap();
    }
}


