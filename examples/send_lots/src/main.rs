#![feature(scoped)]
#![allow(unused_variables)]
extern crate env_logger;
extern crate stomp;
use stomp::header::{ContentType};
use stomp::subscription::AckOrNack::Ack;
use stomp::frame::Frame;
use std::thread;
use std::process;

const TOTAL_MESSAGES : u64 = 1_000_000;
const INTERVAL : u64 = 1000;

fn main() {
  env_logger::init().unwrap();

  let destination = "/queue/sullivan";
  let mut messages_received: u64 = 0;

  let mut subscribe_session = match stomp::session("127.0.0.1", 61613)
    .start() {
      Ok(session) => session,
      Err(error)  => panic!("Could not connect to the server: {}", error)
    };

  let subscription = subscribe_session.subscription(destination, |_: &Frame| {
    messages_received += 1;
    if messages_received % INTERVAL == 0 {
      println!("{} messages received ", messages_received);
    }
    if messages_received >= TOTAL_MESSAGES {
      println!("Receive complete.");
      process::exit(0);
    }
    Ack
  }).start();

  let join_guard = thread::scoped(move || {
    let mut messages_sent: u64 = 0;
    let mut publish_session = match stomp::session("127.0.0.1", 61613)
      .start() {
        Ok(session) => session,
        Err(error)  => panic!("Could not connect to the server: {}", error)
      };
    loop {
      let _ = publish_session.message(destination, "Modern major general")
      .with(ContentType("text/plain"))
      .send();
      messages_sent += 1;
      if messages_sent % INTERVAL == 0 {
        println!("{} messages sent", messages_sent);
      }
      if messages_sent >= TOTAL_MESSAGES {
        println!("Send complete.");
        break;
      }
    }
    let _ = publish_session.disconnect();
    println!("Disconnected.");
  });

  let _ = subscribe_session.listen(); // Loops infinitely, awaiting messages

  let _ = subscribe_session.disconnect();
}
