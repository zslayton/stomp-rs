use std::io::net::tcp::TcpStream;
use std::io::BufferedReader;
use std::io::IoResult;
use std::io::IoError;
use std::io::InvalidInput;
use std::str::from_utf8;
use frame::Frame;
use headers::Header;
use headers::HeaderList;

pub struct Connection {
  ip_address : String,
  port: u16,
  writer  : TcpStream,
  reader  : BufferedReader<TcpStream>
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

  fn send_connect_frame(&mut self) -> IoResult<()> {
    let mut header_list : HeaderList = HeaderList::with_capacity(2);
    header_list.push(Header::from_str("accept-version:1.2").unwrap());
    header_list.push(Header::from_str("content-length:0").unwrap());
    let connect_frame = Frame {
       command : "CONNECT".to_string(),
       headers : header_list,
       body : Vec::new() 
    };
    connect_frame.write(&mut self.writer)
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

  // This method should return a STOMP session rather than the
  // CONNECT frame. The session should hold the Session ID
  // and the connection
  pub fn connect(&mut self) -> IoResult<Frame> {
    let _ = self.send_connect_frame(); // Handle this frame
    self.read_connected_frame()
  }
}
