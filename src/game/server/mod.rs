use std::{io::{Read, Write}, net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream}, };

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{CLOSING_REQUESTED};

pub(super) const SERVER_SOCKET_ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7880);
pub(crate) static CONFIG: bincode_next::config::Configuration<
    bincode_next::config::BigEndian, 
    bincode_next::config::Fixint,
    bincode_next::config::NoLimit, 
    bincode_next::config::SkipBitPacking, 
    bincode_next::config::MsbFirst, 
> = bincode_next::config::standard()
    .with_big_endian()
    .with_fixed_int_encoding()
    .with_no_bit_packing();

pub trait ServerLoopBody<State, FromServer, ToServer> = FnMut(&mut State, ToServer) -> FromServer + Send + 'static;

pub(super) fn init_server_loop<State, FromServer, ToServer>(
    body: impl ServerLoopBody<State, FromServer, ToServer>, 
    state: State,
    socket: std::net::SocketAddr
) where 
    State: Send + 'static,
    FromServer: Serialize,
    ToServer: DeserializeOwned + Default,
{
    let _ = std::thread::spawn(move || {
        server_loop::<State, FromServer, ToServer>(body, state, socket);
    });
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick(u8);

impl Tick {
    pub fn advance(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }

    pub fn get(&self) -> u8 {
        self.0
    }

    pub fn new() -> Self {
        Self(0)
    }
}

fn server_loop<State, FromServer: Serialize, ToServer: DeserializeOwned + Default>(
    mut body: impl ServerLoopBody<State, FromServer, ToServer>, 
    mut state: State,
    socket: std::net::SocketAddr,
) {
    log::info!("statr");
    // std::thread::sleep(std::time::Duration::from_millis(500));
    // let mut buf = [0u8; 2048];

    // let mut stream = utils::try_result_until_success(|| TcpStream::connect(SERVER_SOCKET_ADDRESS));
    let listener = TcpListener::bind(socket).unwrap();
    // let mut stream = TcpStream::connect(socket).unwrap();
    // thread::sleep(Duration::from_millis(200));
    log::info!("Blocking until Tcp connection...");
    let (mut stream, _) = listener.accept().unwrap();
    let mut to_server = ToServer::default();
    log::info!("Connected!");
    // let mut current = Tick(0);
    loop {
        if CLOSING_REQUESTED.load(std::sync::atomic::Ordering::Acquire) { break; }
        // write_to_stream(&mut stream, current.clone()).unwrap();
        // log::info!("{input:?}");
        let from_server = body(&mut state, to_server);
        write_to_stream(&mut stream, from_server).unwrap();

        // current.advance();

        // break;
        // sleep(Duration::from_millis(1000));

        to_server = read_from_stream(&mut stream).unwrap();

    }

    log::info!("Server main loop exiting now.");
}



pub(super) fn read_from_stream<Data: DeserializeOwned>(stream: &mut TcpStream) -> std::io::Result<Data> {
    let mut length = [0u8; 4];
    stream.read_exact(&mut length).unwrap();
    let length = u32::from_be_bytes(length) as usize;
    let mut buf = vec![0u8; length];
    stream.read_exact(&mut buf).unwrap();
    log::debug!("buffer after read: {:?}", buf);
    Ok(bincode_next::serde::decode_from_slice(&buf, CONFIG).unwrap().0)
}

pub(super) fn write_to_stream<Data: Serialize>(stream: &mut TcpStream, data: Data) -> std::io::Result<()> {
    
    let mut msg = [0u8; 1024];
    let length = bincode_next::serde::encode_into_slice(data, &mut msg, CONFIG).unwrap();
    log::debug!("{length} bytes to be sent");
    stream.write_all(&(length as u32).to_be_bytes())?;
    stream.write_all( &msg[0..length])?;
    Ok(())
}