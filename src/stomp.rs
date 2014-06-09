#![crate_id = "stomp#0.0.1"]
#![crate_type = "lib"]
#![desc = "A STOMP 1.2 client implementation in Rust."]
#![license = "MIT"]

use std::io::IoResult;
use session::Session;
use connection::Connection;

pub fn connect(ip_address: &str, port: u16) -> IoResult<Session> {
  let connection = try!(Connection::new(ip_address, port));
  connection.start_session()
}

pub mod headers;
pub mod connection;
pub mod session;
pub mod frame;
