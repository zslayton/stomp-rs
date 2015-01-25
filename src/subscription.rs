use frame::Frame;

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
  pub fn new<S>(id: u32, topic: &str, ack_mode: AckMode, handler: S) -> Subscription <'a> where S : MessageHandler + 'a {
    Subscription {
      id: format!("stomp-rs/{}",id),
      topic: topic.to_string(),
      ack_mode: ack_mode,
      handler: Box::new(handler)
    }
  }
}
