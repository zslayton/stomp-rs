extern crate env_logger;
extern crate stomp;
extern crate tokio;
#[macro_use]
extern crate futures;

use futures::future::join_all;
use stomp::connection::{Credentials, HeartBeat};
use stomp::frame::Frame;
use stomp::header::{Destination, Header, SuppressedHeader};
use stomp::session::{Session, SessionEvent};
use stomp::session_builder::SessionBuilder;
use stomp::subscription::AckMode;
use tokio::prelude::*;

struct ExampleSession {
    session: Session,
    destination: String,
    session_number: u32,

    subscription_id: Option<String>,
}

impl ExampleSession {
    fn on_connected(&mut self) {
        println!("Example session established.");
        let destination = &self.destination;
        println!("Subscribing to '{}'.", destination);

        let subscription_id = self
            .session
            .subscription(destination)
            .with(AckMode::Auto)
            .with(Header::new("custom-subscription-header", "lozenge"))
            .start();

        self.subscription_id = Some(subscription_id);

        self.session.message(destination, "Animal").send();
        self.session.message(destination, "Vegetable").send();
        self.session.message(destination, "Mineral").send();
    }

    fn on_gilbert_and_sullivan_reference(&mut self, frame: Frame) {
        println!(
            "Another droll reference!: '{}'",
            std::str::from_utf8(&frame.body).expect("Non-utf8 bytes")
        );
    }
}

impl Future for ExampleSession {
    type Item = ();
    type Error = std::io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let msg = try_ready!(self.session.poll());
        match msg {
            None => {
                return Ok(Async::Ready(()));
            }
            Some(msg) => {
                match msg {
                    SessionEvent::Connected => {
                        self.on_connected();
                    }

                    SessionEvent::Receipt {
                        id: _id,
                        original,
                        receipt,
                    } => {
                        let Destination(destination) = original.headers.get_destination().unwrap();
                        println!(
                            "Received a Receipt for our subscription to '{}':\n{}",
                            destination, receipt
                        );
                    }

                    SessionEvent::Message {
                        destination: _destination,
                        ack_mode: _ack_mode,
                        frame,
                    } => {
                        let subscribed =
                            if let Some(subscription) = frame.headers.get_subscription() {
                                self.subscription_id == Some(subscription.0.to_owned())
                            } else {
                                false
                            };

                        if subscribed {
                            self.on_gilbert_and_sullivan_reference(frame)
                        }
                    }

                    SessionEvent::SubscriptionlessFrame(_frame) => {
                        //
                    }
                    SessionEvent::UnknownFrame(_frame) => {
                        //
                    }

                    SessionEvent::ErrorFrame(frame) => {
                        println!("Something went horribly wrong: {}", frame);
                    }

                    SessionEvent::Disconnected(reason) => {
                        println!(
                            "Session #{} disconnected, reason={:?}",
                            self.session_number, reason
                        );
                    }
                }
            }
        }
        Ok(Async::NotReady)
    }
}

fn main() {
    env_logger::init();

    println!("Setting up client.");
    let mut sessions = Vec::new();
    for session_number in 0..1 {
        let session = SessionBuilder::new("127.0.0.1", 61613)
            .with(Header::new("custom-client-id", "hmspna4"))
            .with(SuppressedHeader("content-length"))
            .with(HeartBeat(5_000, 2_000))
            .with(Credentials("sullivan", "m1k4d0"))
            .start()
            .expect("failed to build session");

        let session = ExampleSession {
            session,
            session_number: 0,
            destination: format!("/topic/modern_major_general_{}", session_number),

            subscription_id: None,
        };
        sessions.push(session);
    }

    println!("Starting session.");
    tokio::run(
        join_all(sessions)
            .map(|_| println!("all sessions are finished"))
            .map_err(|e| {
                println!("error: {:?}", e);
            }),
    );
}
