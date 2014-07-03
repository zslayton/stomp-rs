use std::io::net::tcp::TcpStream;
use std::io::BufferedReader;
use std::io::IoResult;
use std::io::IoError;
use std::io::InvalidInput;
use std::str::from_utf8;
use frame::Frame;
use session::Session;

pub struct Connection {
  pub ip_address : String,
  pub port: u16,
  pub writer  : TcpStream,
  pub reader  : BufferedReader<TcpStream>
}

impl Connection {
  pub fn new(ip_address: &str, port: u16) -> IoResult<Connection> {
    let stream = try!(TcpStream::connect(ip_address, port));
    Ok(Connection {
      ip_address: ip_address.to_string(),
      port: port,
      writer : stream.clone(),
      reader : BufferedReader::new(stream)
    })
  }

  fn read_connected_frame(&mut self) -> IoResult<Frame> {
    let frame : Frame = try!(Frame::read(&mut self.reader));
    match frame.command.as_slice() {
      "CONNECTED" => Ok(frame),
      _ => Err(IoError{
             kind: InvalidInput, 
             desc: "Could not connect.",
             detail: from_utf8(frame.body.as_slice()).map(|err: &str| err.to_string())
           })
    }
  }

  pub fn start_session(mut self) -> IoResult<Session> {
    let connect_frame = Frame::connect();
    let _ = try!(connect_frame.write(&mut self.writer));
    let _ = try!(self.read_connected_frame());
    Ok(Session::new(self))
  }
}
