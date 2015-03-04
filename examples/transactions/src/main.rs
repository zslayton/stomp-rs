extern crate env_logger;
extern crate stomp;
use stomp::frame::Frame;
use stomp::header::{Header, SuppressedHeader, ContentType};
use stomp::subscription::AckOrNack::Ack;
use stomp::subscription::AckMode;
use stomp::connection::{HeartBeat, Credentials};
use stomp::session::ReceiptHandler;

fn main() {
  env_logger::init().unwrap();

  let destination = "/topic/messages";
  let mut message_count: u64 = 0;

  let mut session = match stomp::session("127.0.0.1", 61613).start() {
     Ok(session) => session,
     Err(error)  => panic!("Could not connect to the server: {}", error)
   };

  let sub_id = session.subscription(destination, |frame: &Frame| {
    message_count += 1;
    println!("Received message #{}:\n{}", message_count, frame);
    Ack
  })
  .start();

  match session.begin_transaction() {
    Ok(mut transaction) => {    
      transaction.message(destination, "Animal").send();
      transaction.message(destination, "Vegetable").send();
      transaction.message(destination, "Mineral").send();
      transaction.commit();
    },
    Err(error)  => panic!("Could not connect to the server: {}", error)
  };

  session.listen(); // Loops infinitely, awaiting messages

  session.disconnect();
}
