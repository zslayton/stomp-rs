#![crate_name = "stomp"]
#![crate_type = "lib"]
#![feature(phase)]

#[phase(plugin, link)]
extern crate log;
extern crate collections;

use std::io::IoResult;
use session::Session;
use connection::Connection;

pub fn connect(ip_address: &str, port: u16) -> IoResult<Session> {
  connect_with_heartbeat(ip_address, port, 0, 0)
}

pub fn connect_with_heartbeat(ip_address: &str, port: u16, tx_heartbeat_ms: uint, rx_heartbeat_ms: uint) -> IoResult<Session> {
  let connection = try!(Connection::new(ip_address, port));
  connection.start_session(tx_heartbeat_ms, rx_heartbeat_ms)
}

pub fn connect_with_credentials(ip_address: &str, port: u16, login: &str, passcode: &str) -> IoResult<Session> {
  connect_with_credentials_and_heartbeat(ip_address, port, login, passcode, 0, 0)
}

pub fn connect_with_credentials_and_heartbeat(ip_address: &str, port: u16, login: &str, passcode: &str, tx_heartbeat_ms: uint, rx_heartbeat_ms: uint) -> IoResult<Session> {
  let connection = try!(Connection::new(ip_address, port));
  connection.start_session_with_credentials(login, passcode, tx_heartbeat_ms, rx_heartbeat_ms)
}

pub mod connection;
pub mod frame;
pub mod header;
pub mod session;
pub mod subscription;
pub mod transaction;
