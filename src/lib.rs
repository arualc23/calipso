#![feature(vec_from_fn)]
#![feature(trait_alias)]
#![warn(rustdoc::broken_intra_doc_links)]


use egui_winit::winit::{self, platform::wayland::EventLoopBuilderExtWayland};

pub mod window;
pub mod map;
pub mod utils;
pub mod game;
pub mod unit_display;
pub mod tile;
pub mod id;

pub mod consts;

pub(crate) use window::CLOSING_REQUESTED;
pub use consts::*;
pub use utils::threads::{THREAD_POOL, PROCESS_COUNT};

#[inline]
pub(crate) fn global_close() {
    CLOSING_REQUESTED.store(true, std::sync::atomic::Ordering::Release);
}

pub fn init(initializer: impl FnOnce(&window::EguiRenderer) -> Box<dyn window::state::State>, title: String) {
    let _ = *PROCESS_COUNT;
    #[cfg(test)]
    let event_loop = winit::event_loop::EventLoop::builder().with_any_thread(true).build().unwrap();
    #[cfg(not(test))]
    let event_loop = winit::event_loop::EventLoop::builder().with_any_thread(false).build().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = window::ControlFlow::new(initializer, title);

    event_loop.run_app(&mut app).unwrap();
}

#[cfg(test)]
mod tests {

// use crate::{unit_display::UnitDisplay};

    // use core::panicking::panic;

use std::net::{TcpListener, TcpStream};

use super::*;
#[allow(unused_imports)]
    use log::{debug, error, info, warn};

    #[test]
    fn thread_pool() {
        env_logger::init();
        let _ = *PROCESS_COUNT;

        let now = std::time::Instant::now();
        THREAD_POOL.lock().unwrap().scope(|s| {
            for _ in 0..*PROCESS_COUNT {
                s.add_job(|| dummy_fn());
            }
        });

        log::info!("{}", now.elapsed().as_millis());

    }

    #[inline(never)]
    fn dummy_fn() {
        const BIG: usize = 100000;
        let mut x: f64 = 1.1;
        let random = rand::random::<u8>() as f64 / 1280000000.0;
        let mut fin = vec![0.0; BIG * 100];
        for j in 0..100 {
            let mut res = vec![0.0; BIG];
            for i in 0..BIG {
                x = x.powf(1.00000001 + random);
                res[i] = x;
            }
            fin[BIG*j..BIG*(j+1)].copy_from_slice(&res);
        }
        


        log::info!("{x}");
    }

    // #[test]
    // pub(crate) fn test1() {
    //     env_logger::init();
    //     info!("Staritng test");
    //     init(|renderer| {
    //         let (map, lmap) = map::load_map("test1", renderer.ctx().clone()).unwrap();
    //         Box::new(TestState::new(map, lmap))
    //     }, "Test".to_string());
    //     info!("After init");
    // }

    // struct TestState {
    //     map: map::Map,
    //     units: Vec<unit_display::UnitDisplay>,
    // }

    // impl window::state::State for TestState {
    //     fn run_frame(&mut self, ui: &mut egui_winit::egui::Ui) {

    //         let painter = ui.layer_painter(*BACKGROUND_LAYER);
    //         self.map.run_frame(ui, Some(&painter), &self.units);
    //     }
    //     fn transition(&mut self) -> Option<Box<dyn window::state::State>> {
    //         None
    //     }
    // }

    // impl TestState {
    //     pub fn new(map: map::Map, logical_map: game::LogicalMap) -> Self {
    //         let unit_texture = utils::load_texture_from_path("assets/textures/unit.png", &map.ctx(), "unit", utils::TextureOptions::Smooth).unwrap();

    //         let send = game::interface::init_game_loop(|(_input, _msg): &(_, ())| info!("Hello world!"), logical_map, map.get_raw_image());
    //         let _ = Box::leak(Box::new(send)); //so that main loop doesn't exit immediately

    //         Self { map, units: vec![UnitDisplay::new(0.into(), unit_texture)] }
    //     }
        
    // }

    // #[test]
    // fn test2() {
    //     // panic!();
    //     env_logger::init();
    //     log::info!("hello");
    //     println!("hello");
    //     game::server::init_server_loop(|a| {}, (), game::server::SERVER_SOCKET_ADDRESS);
    // }

    // #[test]
    // fn test3() {
    //     use std::io::{Read, Write};
        
    //     let mut listener = TcpListener::bind("127.0.0.1:7891").unwrap();
    //     let mut stream_in = TcpStream::connect("127.0.0.1:7891").unwrap();
    //     let (mut stream_out, _) = listener.accept().unwrap();
    //     let mut buf = [0u8; 16];
    //     let length = bincode_next::serde::encode_into_slice((), &mut buf, game::server::CONFIG).unwrap();
    //     stream_in.write_all(&(length as u32).to_be_bytes());
    //     stream_in.write_all( &buf[0..length]);


    //     let mut length = [0u8; 4];
    //     stream_out.read_exact(&mut length).unwrap();
    //     let length = u32::from_be_bytes(length) as usize;
    //     let mut buf = vec![0u8; length];
    //     stream_out.read_exact(&mut buf).unwrap();
    //     log::debug!("buffer after read: {:?}", buf);
    //     let _ = Ok(bincode_next::serde::decode_from_slice(&buf, game::server::CONFIG).unwrap().0);
    // }

    // #[test]
    // fn test4() {
    //     env_logger::init();
    //     let (mut writer, mut reader) = utils::dbuffer::new(0u32);

    //     let _ = std::thread::spawn(move || {let mut i = 1; loop {writer.write(i); writer.swap(); i+= 1; std::thread::sleep(std::time::Duration::from_millis(100));}});
    //     loop {
    //         log::info!("{}", reader.read());
    //         std::thread::sleep(std::time::Duration::from_millis(100));
    //     }
    // }

}