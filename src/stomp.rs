#![crate_id = "stomp#0.0.1"]
#![crate_type = "lib"]
#![desc = "A STOMP 1.2 client implementation in Rust."]
#![license = "MIT"]
#![feature(phase)]

#[phase(plugin, link)]
extern crate log;

use std::io::IoResult;
use session::Session;
use connection::Connection;

pub fn connect(ip_address: &str, port: u16) -> IoResult<Session> {
  let connection = try!(Connection::new(ip_address, port));
  connection.start_session()
}

pub mod connection;
pub mod frame;
pub mod headers;
pub mod session;
pub mod subscription;
pub mod transaction;
