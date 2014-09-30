use std::io::net::tcp::TcpStream;
use std::io::IoResult;
use std::io::IoError;
use std::io::InvalidInput;
use std::str::from_utf8;
use frame::Frame;
use session::Session;
use header::Header;

pub struct Connection {
  pub ip_address : String,
  pub port: u16,
  pub tcp_stream : TcpStream
}

impl Connection {

  pub fn new(ip_address: &str, port: u16) -> IoResult<Connection> {
    let tcp_stream = try!(TcpStream::connect(ip_address, port));
    Ok(Connection {
      ip_address: ip_address.to_string(),
      port: port,
      tcp_stream: tcp_stream
    })
  }

  pub fn start_session(self, tx_heartbeat_ms: uint, rx_heartbeat_ms: uint) -> IoResult<Session> {
    let session = Session::new(self, tx_heartbeat_ms, rx_heartbeat_ms);
    let connect_frame = Frame::connect(tx_heartbeat_ms, rx_heartbeat_ms);
    //let _ = try!(connect_frame.write(&mut self.sender));
    session.send(connect_frame);
    let connected_frame = session.receive();
    match connected_frame.command.as_slice() {
      "CONNECTED" => debug!("Received CONNECTED frame: {}", connected_frame),
      _ => return Err(IoError{
             kind: InvalidInput, 
             desc: "Could not connect.",
             detail: from_utf8(connected_frame.body.as_slice()).map(|err: &str| err.to_string())
           })
    };
    Ok(session)
  }

  pub fn start_session_with_credentials(self, login: &str, passcode: &str, tx_heartbeat_ms: uint, rx_heartbeat_ms: uint) -> IoResult<Session> {
    let session = Session::new(self, tx_heartbeat_ms, rx_heartbeat_ms);
    let mut connect_frame = Frame::connect(tx_heartbeat_ms, rx_heartbeat_ms);
    connect_frame.headers.push(
      Header::encode_key_value("login", login)
    );
    connect_frame.headers.push(
      Header::encode_key_value("passcode", passcode)
    );
    session.send(connect_frame);
    let connected_frame = session.receive();
    match connected_frame.command.as_slice() {
      "CONNECTED" => debug!("Received CONNECTED frame: {}", connected_frame),
      _ => return Err(IoError{
             kind: InvalidInput, 
             desc: "Could not connect.",
             detail: from_utf8(connected_frame.body.as_slice()).map(|err: &str| err.to_string())
           })
    };
    Ok(session)
  }
}
