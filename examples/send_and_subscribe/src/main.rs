extern crate stomp;
use stomp::frame::Frame;
use stomp::subscription::AckOrNack::{self, Ack};
use stomp::subscription::AckMode::Client;
use stomp::subscription::MessageHandler;

use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::Thread;

fn main() {
  let mut session = match stomp::connect("127.0.0.1", 61613) {
    Ok(session)  => session,
    Err(error) => panic!("Could not connect to the server: {}", error)
  };
  
  let topic = "/topic/messages";
  let mut message_count : u64 = 0;
  session.subscribe(topic, Client, |&mut: frame: &Frame|{
    message_count += 1;
    println!("Closure -> Received message #{}:\n{}", message_count, frame);
    Ack
  }); // 'client' acknowledgement mode

  // Send arbitrary bytes with a specified MIME type
  session.send_bytes(topic, "text/plain", "Animal".as_bytes());

  // Send UTF-8 text with an assumed MIME type of 'text/plain'
  session.send_text(topic, "Vegetable");
  session.send_text(topic, "Mineral");

  
  session.listen(); // Loops infinitely, awaiting messages

  session.disconnect();
}
