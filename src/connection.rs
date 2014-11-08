use std::io::net::tcp::TcpStream;
use std::io::BufferedReader;
use std::io::BufferedWriter;
use frame::HeartBeat;
use frame::CompleteFrame;
use std::io::IoResult;
use std::io::IoError;
use std::io::InvalidInput;
use std::str::from_utf8;
use frame::Frame;
use session::Session;
use std::cmp::max;
use header;
use header::Header;
use header::StompHeaderSet;

pub struct Connection {
  pub ip_address : String,
  pub port: u16,
  pub tcp_stream : TcpStream
}

impl Connection {

  pub fn new(ip_address: &str, port: u16) -> IoResult<Connection> {
    let tcp_stream = try!(TcpStream::connect((ip_address, port)));
    Ok(Connection {
      ip_address: ip_address.to_string(),
      port: port,
      tcp_stream: tcp_stream
    })
  }

  fn select_heartbeat(client_tx_ms:uint, client_rx_ms:uint, server_tx_ms:uint, server_rx_ms:uint) -> (uint, uint) {
    let heartbeat_tx_ms: uint;
    let heartbeat_rx_ms: uint;
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

  pub fn start_session(mut self, client_tx_ms: uint, client_rx_ms: uint) -> IoResult<Session> {
    let connect_frame = Frame::connect(client_tx_ms, client_rx_ms);
    let (server_tx_ms, server_rx_ms) = try!(self.start_session_with_frame(connect_frame));
    let (tx_ms, rx_ms) = Connection::select_heartbeat(client_tx_ms, client_rx_ms, server_tx_ms, server_rx_ms);
    let session = Session::new(self, tx_ms, rx_ms);
    Ok(session)
  }

  pub fn start_session_with_credentials(mut self, login: &str, passcode: &str, client_tx_ms: uint, client_rx_ms: uint) -> IoResult<Session> {
    let mut connect_frame = Frame::connect(client_tx_ms, client_rx_ms);
    connect_frame.headers.push(
      Header::encode_key_value("login", login)
    );
    connect_frame.headers.push(
      Header::encode_key_value("passcode", passcode)
    );
    let (server_tx_ms, server_rx_ms) = try!(self.start_session_with_frame(connect_frame));
    let (tx_ms, rx_ms) = Connection::select_heartbeat(client_tx_ms, client_rx_ms, server_tx_ms, server_rx_ms);
    let session = Session::new(self, tx_ms, rx_ms);
    Ok(session)
  }
  
  pub fn start_session_with_frame(&mut self, connect_frame: Frame) -> IoResult<(uint, uint)> {
    let mut buffered_writer = BufferedWriter::new(self.tcp_stream.clone());
    try!(connect_frame.write(&mut buffered_writer));
    let connected_frame : Frame;
    let mut buffered_reader = BufferedReader::new(self.tcp_stream.clone());
    loop{
      let transmission = try!(Frame::read(&mut buffered_reader));
      match transmission {
        HeartBeat => continue,
        CompleteFrame(frame) => {
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
             detail: from_utf8(connected_frame.body.as_slice()).map(|err: &str| err.to_string())
           })
    }
    match connected_frame.headers.get_heart_beat() {
      Some(header::HeartBeat(tx_ms, rx_ms)) => Ok((tx_ms, rx_ms)),
      None => Ok((0, 0))
    }
  }
}
