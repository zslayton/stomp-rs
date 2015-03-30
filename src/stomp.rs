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

extern crate mio;

use mio::*;

use session_builder::SessionBuilder;

pub fn session<'a>(host: &'a str, port: u16) -> SessionBuilder<'a>{
  SessionBuilder::new(host, port)
}

enum StompTimeout {
  SendHeartBeat,
  ReceiveHeartBeat
}

struct StompHandler;

const SEND_HEART_BEAT_MS: u64 = 2_000;
const RECEIVE_HEART_BEAT_MS: u64 = 6_000;

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
}

pub fn event_loop() {
  let mut event_loop = EventLoop::new().unwrap();
  let timeout = event_loop.timeout_ms(StompTimeout::SendHeartBeat, 2000).unwrap();
  let timeout = event_loop.timeout_ms(StompTimeout::ReceiveHeartBeat, 6000).unwrap();
  let _ = event_loop.run(&mut StompHandler);
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
