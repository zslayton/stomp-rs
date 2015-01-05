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

pub struct Subscription {
  pub id : String,
  pub topic: String,
  pub ack_mode: AckMode,
  pub callback: fn(&Frame)-> AckOrNack
}

impl Subscription {
  pub fn new(id: uint, topic: &str, ack_mode: AckMode, callback: fn(&Frame)->AckOrNack) -> Subscription {
    Subscription {
      id: format!("stomp-rs/{}",id),
      topic: topic.to_string(),
      ack_mode: ack_mode,
      callback: callback 
    }
  }
}
