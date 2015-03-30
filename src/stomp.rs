#![crate_name = "stomp"]
#![crate_type = "lib"]

#![feature(collections)]
#![feature(core)]
#![feature(std_misc)]
#![feature(io)]
#![feature(old_io)]
#![feature(slice_patterns)]

#[macro_use]
extern crate log;
extern crate collections;

extern crate core;
extern crate mio;

use mio::*;
use mio::tcp::*;

use session_builder::SessionBuilder;

use frame::Frame;
use std::io::Read;
use core::ops::DerefMut;

pub fn session<'a>(host: &'a str, port: u16) -> SessionBuilder<'a>{
  SessionBuilder::new(host, port)
}

enum StompTimeout {
  SendHeartBeat,
  ReceiveHeartBeat
}

const SEND_HEART_BEAT_MS: u64 = 2_000;
const RECEIVE_HEART_BEAT_MS: u64 = 6_000;

struct StompHandler(NonBlock<TcpStream>);

impl Handler for StompHandler{
  type Timeout = StompTimeout;
  type Message = ();

  fn timeout(&mut self, event_loop: &mut EventLoop<StompHandler>, mut timeout: StompTimeout) {
    match timeout {
      StompTimeout::SendHeartBeat => {
        println!("Send heartbeat");
        event_loop.timeout_ms(StompTimeout::SendHeartBeat, ::SEND_HEART_BEAT_MS);
      },
      StompTimeout::ReceiveHeartBeat => {
        println!("Receive heartbeat");
        event_loop.timeout_ms(StompTimeout::ReceiveHeartBeat, ::RECEIVE_HEART_BEAT_MS);
      },
    }
  }

  fn readable(&mut self, event_loop: &mut EventLoop<StompHandler>, token: Token, _: ReadHint) {
    println!("Readable!");
    let mut response = String::new();
    match *self {
      StompHandler(ref mut sock) => {
        sock.read_to_string(&mut response);
        println!("Response:\n{}", response);
      }
    } 
    event_loop.shutdown();
  }
}

pub fn event_loop() {
  let mut event_loop = EventLoop::new().unwrap();
  let addr = "127.0.0.1:61613".parse().unwrap();
  let (mut sock, _) = tcp::connect(&addr).unwrap();
  Frame::connect(SEND_HEART_BEAT_MS as u32, RECEIVE_HEART_BEAT_MS as u32).write(sock.deref_mut());
  let _ = event_loop.timeout_ms(StompTimeout::SendHeartBeat, 2000).unwrap();
  let _ = event_loop.timeout_ms(StompTimeout::ReceiveHeartBeat, 6000).unwrap();
  event_loop.register(&sock, Token(0)).unwrap();
  let mut handler = StompHandler(sock);
  let _ = event_loop.run(&mut handler); 
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
