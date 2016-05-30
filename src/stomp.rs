#![crate_name = "stomp"]
#![crate_type = "lib"]


#[macro_use]
extern crate log;
extern crate mio;
extern crate mai;
extern crate lifeguard;
extern crate unicode_segmentation;

use session::Client; //TODO: Make new module for Client

pub fn client<H>() -> Client<H> where H: handler::Handler + 'static {
    Client::new()
}

pub mod handler;
pub mod connection;
pub mod header;
pub mod codec;
pub mod frame;
pub mod session;
pub mod session_manager;
pub mod subscription;
pub mod transaction;
pub mod message_builder;
pub mod protocol;
pub mod session_builder;
pub mod subscription_builder;
pub mod timeout;
pub mod option_setter;
