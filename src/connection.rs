use std::io::net::tcp::TcpStream;
use std::io::BufferedReader;
use std::io::IoResult;
use std::io::IoError;
use std::io::InvalidInput;
use std::str::from_utf8;
use frame::Frame;
use headers::Header;
use headers::HeaderList;

struct Connection {
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

  fn send_connect_frame(&mut self) {
    let mut header_list : HeaderList = HeaderList::new(1);
    header_list.push(Header::from_str("accept-version:1.2").unwrap());
    let mut connect_frame = Frame {
       command : "CONNECT".to_string(),
       headers : header_list,
       body : Vec::new() 
    };
    connect_frame.write(&mut self.writer);
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

  pub fn connect(&mut self) -> IoResult<Frame> {
    self.send_connect_frame();
    self.read_connected_frame()
  }
}
