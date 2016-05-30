use frame::Frame;
use subscription::AckOrNack::{Ack, Nack};
use header::HeaderList;
use handler::Handler;
use session::Session;
use std::sync::mpsc::Sender;

#[derive(Copy,Clone)]
pub enum AckMode {
    Auto,
    Client,
    ClientIndividual,
}

impl AckMode {
    pub fn as_text(&self) -> &'static str {
        match *self {
            AckMode::Auto => "auto",
            AckMode::Client => "client",
            AckMode::ClientIndividual => "client-individual",
        }
    }
}

#[derive(Clone, Copy)]
pub enum AckOrNack {
    Ack,
    Nack,
}

pub struct Subscription<H> where H: Handler {
    pub id: String,
    pub destination: String,
    pub ack_mode: AckMode,
    pub headers: HeaderList,
    pub handler: MessageHandler<H>,
}

pub type MessageHandler<H> = fn(&mut H, &mut Session<H>, &Frame) -> AckOrNack;

impl <H> Subscription<H> where H: Handler {
    pub fn new(id: u32,
               destination: &str,
               ack_mode: AckMode,
               headers: HeaderList,
               message_handler: MessageHandler<H>)
               -> Subscription<H> {
        Subscription {
            id: format!("stomp-rs/{}", id),
            destination: destination.to_string(),
            ack_mode: ack_mode,
            headers: headers,
            handler: message_handler,
        }
    }
}
