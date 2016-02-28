#![crate_name = "stomp"]
#![crate_type = "lib"]


#[macro_use]
extern crate log;
extern crate mio;
extern crate mai;
extern crate lifeguard;
extern crate unicode_segmentation;

use session_builder::SessionBuilder;

pub fn session<'a>(host: &'a str, port: u16) -> SessionBuilder<'a>{
  SessionBuilder::new(host, port)
}

pub mod connection;
pub mod header;
pub mod frame;
pub mod frame_buffer;
pub mod session;
pub mod subscription;
pub mod transaction;
pub mod message_builder;
pub mod session_builder;
pub mod subscription_builder;
pub mod option_setter;
