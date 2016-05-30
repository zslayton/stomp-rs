#[macro_use]
extern crate env_logger;
extern crate stomp;
use stomp::frame::Frame;
use stomp::header::{Header, SuppressedHeader};
use stomp::subscription::AckOrNack::{self, Ack};
use stomp::subscription::AckMode;
use stomp::connection::{HeartBeat, Credentials};
use stomp::session::ReceiptHandler;
use stomp::session::Session;

struct ExampleSession {
    session_number: u32,
    destination: String
}

impl ExampleSession {
    fn new(session_number: u32) -> ExampleSession {
        ExampleSession {
            session_number: session_number,
            destination: format!("topics/modern_major_general_{}", session_number)
        }
    }
}

impl ExampleSession {
    fn on_gilbert_and_sullivan_reference(&mut self, _session: &mut Session<Self>, frame: &Frame) -> AckOrNack {
        println!("Another droll reference!: '{}'", std::str::from_utf8(&frame.body).expect("Non-utf8 bytes"));
        Ack
    }
}

impl stomp::handler::Handler for ExampleSession {
    fn on_connected(&mut self, session: &mut Session<Self>, _frame: &Frame) {
        println!("Example session established.");
        let destination = &self.destination;
        println!("Subscribing to '{}'.", destination);
        let _ = session.subscription(destination, Self::on_gilbert_and_sullivan_reference)
                       .with(AckMode::Auto)
                       .with(Header::new("custom-subscription-header", "lozenge"))
                       .with(ReceiptHandler::new(|_: &Frame| println!("Subscribed successfully.")))
                       .start();

        let _ = session.message(destination, "Animal").send();
        let _ = session.message(destination, "Vegetable").send();
        let _ = session.message(destination, "Mineral").send();
    }

    fn on_receipt(&mut self, _session: &mut Session<Self>, receipt: &Frame) {
        println!("Received a Receipt:\n{}", receipt);
    }

    fn on_error(&mut self, _session: &mut Session<Self>, frame: &Frame) {
        println!("Something went horribly wrong: {}", frame);
    }

    fn on_disconnected(&mut self, _session: &mut Session<Self>) {
        println!("Session #{} disconnected.", self.session_number);
    }
}

fn main() {
    env_logger::init().unwrap();
    println!("Setting up client.");
    let mut client = stomp::client();
    println!("Starting session.");
    for session_number in 0..1 {
        client.session("127.0.0.1", 61613, ExampleSession::new(session_number))
              .with(Header::new("custom-client-id", "hmspna4"))
              .with(SuppressedHeader("content-length"))
              .with(HeartBeat(5_000, 2_000))
              .with(Credentials("sullivan", "m1k4d0"))
              .start();
    }

    println!("Running client.");

    client.run(); // Loops infinitely, processing events
}
