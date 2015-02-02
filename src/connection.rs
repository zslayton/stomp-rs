use std::old_io::net::tcp::TcpStream;
use std::old_io::BufferedReader;
use std::old_io::BufferedWriter;
use frame::Transmission;
use std::old_io::IoResult;
use std::old_io::IoError;
use std::old_io::InvalidInput;
use std::str::from_utf8;
use frame::Frame;
use std::cmp::max;
use header::{self, StompHeaderSet};

pub struct Connection {
  pub ip_address : String,
  pub port: u16,
  pub tcp_stream : TcpStream
}

#[derive(Copy)]
pub struct HeartBeat(pub u32, pub u32);
pub struct Credentials<'a>(pub &'a str, pub &'a str); 
  
impl Connection {

  pub fn new(ip_address: &str, port: u16) -> IoResult<Connection> {
    let tcp_stream = try!(TcpStream::connect((ip_address, port)));
    Ok(Connection {
      ip_address: ip_address.to_string(),
      port: port,
      tcp_stream: tcp_stream
    })
  }

  pub fn select_heartbeat(client_tx_ms:u32, client_rx_ms:u32, server_tx_ms:u32, server_rx_ms:u32) -> (u32, u32) {
    let heartbeat_tx_ms: u32;
    let heartbeat_rx_ms: u32;
    if client_tx_ms == 0 || server_rx_ms == 0 {
      heartbeat_tx_ms = 0; 
    } else {
      heartbeat_tx_ms = max(client_tx_ms, server_rx_ms);
    }
    if client_rx_ms == 0 || server_tx_ms == 0 {
      heartbeat_rx_ms = 0;
    } else {
      heartbeat_rx_ms = max(client_rx_ms, server_tx_ms);
    }
    (heartbeat_tx_ms, heartbeat_rx_ms)
  }

  pub fn start_session_with_frame(&mut self, connect_frame: Frame) -> IoResult<(u32, u32)> {
    let mut buffered_writer = BufferedWriter::new(self.tcp_stream.clone());
    try!(connect_frame.write(&mut buffered_writer));
    let connected_frame : Frame;
    let mut buffered_reader = BufferedReader::new(self.tcp_stream.clone());
    loop{
      let transmission = try!(Frame::read(&mut buffered_reader));
      match transmission {
        Transmission::HeartBeat => continue,
        Transmission::CompleteFrame(frame) => {
          connected_frame = frame;
          break;
        }
      } 
    }
    match connected_frame.command.as_slice() {
      "CONNECTED" => debug!("Received CONNECTED frame: {}", connected_frame),
       _ => return Err(IoError{
             kind: InvalidInput, 
             desc: "Could not connect.",
             detail: from_utf8(connected_frame.body.as_slice()).ok().map(|err: &str| err.to_string())
           })
    }
    match connected_frame.headers.get_heart_beat() {
      Some(header::HeartBeat(tx_ms, rx_ms)) => Ok((tx_ms, rx_ms)),
      None => Ok((0, 0))
    }
  }
}
