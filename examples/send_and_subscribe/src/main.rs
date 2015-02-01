extern crate stomp;
use stomp::frame::Frame;
use stomp::header::Header;
use stomp::subscription::AckOrNack::{self, Ack};
use stomp::subscription::AckMode::Client;
use stomp::subscription::MessageHandler;

fn main() {
  let mut message_count : u64 = 0;
  let on_message = |&mut: frame: &Frame| {
    message_count += 1;
    println!("Received message #{}:\n{}", message_count, frame);
    Ack
  };
  
  let mut session = match stomp::connect("127.0.0.1", 61613) {
    Ok(session)  => session,
    Err(error) => panic!("Could not connect to the server: {}", error)
  };

  let destination = "/topic/messages";
  let acknowledge_mode = Client;
  session.subscribe(destination, acknowledge_mode, on_message);

  // Send arbitrary bytes with a specified MIME type
  session.send_bytes(destination, "text/plain", "Animal".as_bytes());

  // Send UTF-8 text with an assumed MIME type of 'text/plain'
  session.send_text(destination, "Vegetable");
  session.send_text(destination, "Mineral");

  session.message(destination, "text/plain", "Hypoteneuse".as_bytes())
    .with(Header::new("client-id", "0"))
    .with(Header::new("persistent", "true"))
    .send();

  session.listen(); // Loops infinitely, awaiting messages

  session.disconnect();
}
