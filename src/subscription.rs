use frame::Frame;
use subscription::AckOrNack::{Ack, Nack};
use std::sync::mpsc::Sender;

#[derive(Copy)]
pub enum AckMode {
  Auto,
  Client,
  ClientIndividual
}

impl AckMode {
  pub fn as_text(&self) -> &'static str {
    match *self {
      AckMode::Auto => "auto",
      AckMode::Client => "client",
      AckMode::ClientIndividual => "client-individual"
    }
  }
}

#[derive(Copy)]
pub enum AckOrNack {
  Ack,
  Nack
}

pub trait MessageHandler {
  fn on_message(&mut self, &Frame) -> AckOrNack;
}

pub struct Subscription <'a> { 
  pub id : String,
  pub topic: String,
  pub ack_mode: AckMode,
  pub handler: Box<MessageHandler + 'a>
}

impl <'a> Subscription <'a> {
  pub fn new(id: u32, topic: &str, ack_mode: AckMode, message_handler: Box<MessageHandler + 'a>) -> Subscription <'a> {
    Subscription {
      id: format!("stomp-rs/{}",id),
      topic: topic.to_string(),
      ack_mode: ack_mode,
      handler: message_handler
    }
  }
}

pub trait ToMessageHandler{
  fn to_message_handler<'a>(self) -> Box<MessageHandler + 'a>;
}

struct SendingMessageHandler {
  sender: Sender<Frame>
}

impl MessageHandler for SendingMessageHandler {
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
  fn to_message_handler<'a>(self) -> Box<MessageHandler + 'a> {
    Box::new(SendingMessageHandler{sender : self}) as Box<MessageHandler>
  }
}
