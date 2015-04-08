#![crate_name = "stomp"]
#![crate_type = "lib"]

#![feature(collections)]
#![feature(std_misc)]
#![feature(old_io)]
#![feature(slice_patterns)]

#[macro_use]
extern crate log;
extern crate collections;

extern crate core;
extern crate mio;

use session_builder::SessionBuilder;


use mio::*;
use mio::tcp::*;

use frame::Frame;
use std::io::BufReader;
use std::io::Read;
use core::ops::DerefMut;
use frame::Transmission::{HeartBeat, CompleteFrame};

pub fn session<'a>(host: &'a str, port: u16) -> SessionBuilder<'a>{
  SessionBuilder::new(host, port)
}



pub mod connection;
pub mod header;
pub mod frame;
pub mod session;
pub mod subscription;
pub mod transaction;
pub mod message_builder;
pub mod session_builder;
pub mod subscription_builder;
pub mod option_setter;
