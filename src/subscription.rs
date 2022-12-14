use crate::header::HeaderList;

#[derive(Copy, Clone)]
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

pub struct Subscription {
    pub id: String,
    pub destination: String,
    pub ack_mode: AckMode,
    pub headers: HeaderList,
}

impl Subscription {
    pub fn new(id: u32, destination: &str, ack_mode: AckMode, headers: HeaderList) -> Subscription {
        Subscription {
            id: format!("stomp-rs/{}", id),
            destination: destination.to_string(),
            ack_mode: ack_mode,
            headers: headers,
        }
    }
}
