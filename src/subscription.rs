use frame::Frame;
use subscription::AckOrNack::{Ack, Nack};
use header::HeaderList;
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

pub trait MessageHandler: Send {
    fn on_message(&mut self, &Frame) -> AckOrNack;
}

pub struct Subscription {
    pub id: String,
    pub destination: String,
    pub ack_mode: AckMode,
    pub headers: HeaderList,
    pub handler: Box<MessageHandler>,
}

impl Subscription {
    pub fn new(id: u32,
               destination: &str,
               ack_mode: AckMode,
               headers: HeaderList,
               message_handler: Box<MessageHandler>)
               -> Subscription {
        Subscription {
            id: format!("stomp-rs/{}", id),
            destination: destination.to_string(),
            ack_mode: ack_mode,
            headers: headers,
            handler: message_handler,
        }
    }
}

pub trait ToMessageHandler {
    fn to_message_handler(self) -> Box<MessageHandler>;
}

impl<T> ToMessageHandler for T
    where T: MessageHandler + 'static
{
    fn to_message_handler(self) -> Box<MessageHandler> {
        Box::new(self) as Box<MessageHandler>
    }
}

impl ToMessageHandler for Box<MessageHandler> {
    fn to_message_handler(self) -> Box<MessageHandler> {
        self
    }
}
// Support for Sender<T> in subscriptions

struct SenderMessageHandler {
    sender: Sender<Frame>,
}

impl MessageHandler for SenderMessageHandler {
    fn on_message(&mut self, frame: &Frame) -> AckOrNack {
        debug!("Sending frame...");
        match self.sender.send(frame.clone()) {
            Ok(_) => Ack,
            Err(error) => {
                error!("Failed to send frame: {}", error);
                Nack
            }
        }
    }
}

impl ToMessageHandler for Sender<Frame> {
    fn to_message_handler(self) -> Box<MessageHandler> {
        Box::new(SenderMessageHandler { sender: self }) as Box<MessageHandler>
    }
}

impl<F> MessageHandler for F
    where F: Send + FnMut(&Frame) -> AckOrNack
{
    fn on_message(&mut self, frame: &Frame) -> AckOrNack {
        debug!("Passing frame to closure...");
        self(frame)
    }
}
