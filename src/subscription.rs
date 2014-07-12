use frame::Frame;

pub enum AckMode {
  Auto,
  Client,
  ClientIndividual
}

impl AckMode {
  pub fn as_text(&self) -> &'static str {
    match *self {
      Auto => "auto",
      Client => "client",
      ClientIndividual => "client-individual"
    }
  }
}

pub struct Subscription {
  pub id : String,
  pub topic: String,
  pub ack_mode: AckMode,
  pub callback: fn(Frame)-> bool
}

impl Subscription {
  pub fn new(id: uint, topic: &str, ack_mode: AckMode, callback: fn(Frame)->bool) -> Subscription {
    Subscription {
      id: format!("stomp-rs/{}",id),
      topic: topic.to_string(),
      ack_mode: ack_mode,
      callback: callback 
    }
  }
}
