extern crate env_logger;
extern crate stomp;
use stomp::frame::Frame;
use stomp::header::{Header, SuppressedHeader};
use stomp::subscription::AckOrNack::Ack;
use stomp::subscription::AckMode::Client;
use stomp::connection::{HeartBeat, Credentials};

fn main() {
  env_logger::init().unwrap();

  let destination = "/topic/messages";
  let acknowledge_mode = Client;
  let mut message_count: u64 = 0;

  let mut session = match stomp::session("127.0.0.1", 61613)
    .with(Header::new("custom-client-id", "hmspna4"))
    .with(SuppressedHeader("content-length"))
    .with(HeartBeat(5000, 2000))
    .with(Credentials("sullivan", "m1k4d0"))
    .start() {
      Ok(session) => session,
      Err(error)  => panic!("Could not connect to the server: {}", error)
   };

  let sub_id = session.subscription(destination, |&mut: frame: &Frame| {
    message_count += 1;
    println!("Received message #{}:\n{}", message_count, frame);
    Ack
  }).create();

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
