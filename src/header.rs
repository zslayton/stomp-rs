// Non-camel case types are used for Stomp Protocol version enum variants
#![macro_use]
#![allow(non_camel_case_types)]
use collections::slice::Iter;

// Ideally this would be a simple typedef. However:
// See Rust bug #11047: https://github.com/mozilla/rust/issues/11047
// Cannot call static methods (`with_capacity`) on type aliases (`HeaderList`)
#[derive(Clone)]
pub struct HeaderList {
  pub headers: Vec<Header>
}

impl HeaderList {
  pub fn new() -> HeaderList {
    HeaderList::with_capacity(0)
  }
  pub fn with_capacity(capacity: usize) -> HeaderList {
    HeaderList {
      headers: Vec::with_capacity(capacity)
    }
  }

  pub fn push(&mut self, header: Header) {
    self.headers.push(header);
  }

  pub fn iter<'a>(&'a self) -> Iter<'a, Header> {
    self.headers.iter()
  }
}

#[derive(Clone)]
pub struct Header {
  buffer : String,
  delimiter_index : u32
}

impl Header {
  pub fn new(key: &str, value: &str) -> Header {
    Header::encode_key_value(key, value)
  }

  fn from_string(raw_string: &str) -> Option<Header> {
    let delimiter_index = match raw_string.find(':') {
      Some(index) => index,
      None => return None
    };
    let mut header = Header{
      buffer: raw_string.to_string(),
      delimiter_index: delimiter_index as u32
    };
    header = Header::decode_key_value(header.get_key(), header.get_value());
    Some(header)
  }

  pub fn encode_string(raw_string: &str) -> Option<Header> {
    let header = Header::from_string(raw_string);
    header.map(|h| Header::encode_key_value(h.get_key(), h.get_value()))
  }
  
  pub fn decode_string(raw_string: &str) -> Option<Header> {
    let header = Header::from_string(raw_string);
    header.map(|h| Header::decode_key_value(h.get_key(), h.get_value()))
  }

  pub fn encode_key_value(key: &str, value: &str) -> Header {
    let raw_string = format!("{}:{}", key, Header::encode_value(value));
    Header {
      buffer: raw_string,
      delimiter_index: key.len() as u32
    }
  }

  pub fn decode_key_value(key: &str, value: &str) -> Header {
    let raw_string = format!("{}:{}", key, Header::decode_value(value));
    Header {
      buffer: raw_string,
      delimiter_index: key.len() as u32
    }
  }

  //TODO: Optimize this method.
  fn decode_value(value: &str) -> String {
    value
      .replace(r"\c", ":")
      .replace(r"\n", "\n")
      .replace(r"\r", "\r")
      .replace(r"\\", "\\")
  }

//TODO: Optimize this method.
  fn encode_value(value: &str) -> String {
    value
      .replace("\\", r"\\") // Order is significant
      .replace("\r", r"\r")
      .replace("\n", r"\n")
      .replace(":", r"\c")
  }

  pub fn get_raw<'a>(&'a self) -> &'a str {
    self.buffer.as_slice()
  }

  pub fn get_key<'a>(&'a self) -> &'a str {
    let index : usize = self.delimiter_index as usize;
    &self.buffer[..index]
  }

  pub fn get_value<'a>(&'a self) -> &'a str {
    let index : usize = self.delimiter_index as usize + 1;
    &self.buffer[index..]
  }

}

// Headers in the Spec
pub struct AcceptVersion(pub Vec<StompVersion>);
pub struct Ack<'a>(pub &'a str);
#[derive(Copy)]
pub struct ContentLength(pub u32);
pub struct Custom(pub Header);
pub struct Destination<'a> (pub &'a str);
#[derive(Copy)]
pub struct HeartBeat(pub u32, pub u32);
pub struct Host<'a>(pub &'a str);
pub struct Id<'a>(pub &'a str);
pub struct Login<'a>(pub &'a str);
pub struct MessageId<'a>(pub &'a str);
pub struct Passcode<'a>(pub &'a str);
pub struct Receipt<'a>(pub &'a str);
pub struct ReceiptId<'a>(pub &'a str);
pub struct Server<'a>(pub &'a str);
pub struct Session<'a> (pub &'a str);
pub struct Subscription<'a>(pub &'a str);
pub struct Transaction<'a>(pub &'a str);
#[derive(Copy)]
pub struct Version(pub StompVersion);

#[derive(Copy)]
pub enum StompVersion {
  Stomp_v1_0,
  Stomp_v1_1,
  Stomp_v1_2,
}

pub trait StompHeaderSet {
  fn get_content_length(&self) -> Option<ContentLength>;
  fn get_header<'a>(&'a self, key: &str) -> Option<&'a Header>;
  fn get_accept_version<'a>(&'a self) -> Option<Vec<StompVersion>>;
  fn get_ack<'a>(&'a self) -> Option<Ack<'a>>;
  fn get_destination<'a>(&'a self) -> Option<Destination<'a>>;
  fn get_heart_beat(&self) -> Option<HeartBeat>;
  fn get_host<'a>(&'a self) -> Option<Host<'a>>;
  fn get_id<'a>(&'a self) -> Option<Id<'a>>;
  fn get_login<'a>(&'a self) -> Option<Login<'a>>;
  fn get_message_id<'a>(&'a self) -> Option<MessageId<'a>>;
  fn get_passcode<'a>(&'a self) -> Option<Passcode<'a>>;
  fn get_receipt<'a>(&'a self) -> Option<Receipt<'a>>;
  fn get_receipt_id<'a>(&'a self) -> Option<ReceiptId<'a>>;
  fn get_server<'a>(&'a self) -> Option<Server<'a>>;
  fn get_session<'a>(&'a self) -> Option<Session<'a>>;
  fn get_subscription<'a>(&'a self) -> Option<Subscription<'a>>;
  fn get_transaction<'a>(&'a self) -> Option<Transaction<'a>>;
  fn get_version(&self) -> Option<Version>;
}

impl StompHeaderSet for HeaderList {
  
  fn get_header<'a>(&'a self, key: &str) -> Option<&'a Header>{
    self.headers.iter().find(|header| 
      match **header {
        ref h if h.get_key() == key => true, 
        _ => false
      }
    )
  }

  fn get_accept_version(&self) -> Option<Vec<StompVersion>> {
    let versions : &str = match self.get_header("accept-version") {
      Some(h) => h.get_value(),
      None => return None
    };
    let versions: Vec<StompVersion> = versions.split(',').filter_map(|v| match v.trim() {
      "1.0" => Some(StompVersion::Stomp_v1_0),
      "1.1" => Some(StompVersion::Stomp_v1_1),
      "1.2" => Some(StompVersion::Stomp_v1_2),
      _ => None
    }).collect();
    Some(versions)
  }

  fn get_ack<'a>(&'a self) -> Option<Ack<'a>> {
    match self.get_header("ack") {
      Some(h) => Some(Ack(h.get_value())),
      None => None
    }
  }

  fn get_destination<'a>(&'a self) -> Option<Destination<'a>> {
    match self.get_header("destination") {
      Some(h) => Some(Destination(h.get_value())),
      None => return None
    }
  }

  fn get_heart_beat(&self) -> Option<HeartBeat> {
    let spec = match self.get_header("heart-beat") {
      Some(h) => h.get_value(), 
      None => return None
    };
    let spec_list: Vec<u32> = spec.split(',').filter_map(|str_val| str_val.parse::<u32>()).collect();
    match spec_list.as_slice() {
      [x, y] => Some(HeartBeat(x, y)),
      _ => None
    }
  }

  fn get_host<'a>(&'a self) -> Option<Host<'a>> {
    match self.get_header("host") {
      Some(h) => Some(Host(h.get_value())),
      None => None
    }
  }
  
  fn get_id<'a>(&'a self) -> Option<Id<'a>> {
    match self.get_header("id") {
      Some(h) => Some(Id(h.get_value())),
      None => None
    }
  }

  fn get_login<'a>(&'a self) -> Option<Login<'a>> {
    match self.get_header("login"){
      Some(h) => Some(Login(h.get_value())),
      None => None
    }
  }

  fn get_message_id<'a>(&'a self) -> Option<MessageId<'a>> {
    match self.get_header("message-id"){
      Some(h) => Some(MessageId(h.get_value())),
      None => None
    }
  }

  fn get_passcode<'a>(&'a self) -> Option<Passcode<'a>> {
    match self.get_header("passcode"){
      Some(h) => Some(Passcode(h.get_value())),
      None => None
    }
  }

  fn get_receipt<'a>(&'a self) -> Option<Receipt<'a>> {
    match self.get_header("receipt"){
      Some(h) => Some(Receipt(h.get_value())),
      None => None
    }
  }

  fn get_receipt_id<'a>(&'a self) -> Option<ReceiptId<'a>> {
    match self.get_header("receipt-id"){
      Some(h) => Some(ReceiptId(h.get_value())),
      None => None
    }
  }

  fn get_server<'a>(&'a self) -> Option<Server<'a>> {
    match self.get_header("server"){
      Some(h) => Some(Server(h.get_value())),
      None => None
    }
  }

  fn get_session<'a>(&'a self) -> Option<Session<'a>> {
    match self.get_header("session"){
      Some(h) => Some(Session(h.get_value())),
      None => None
    }
  }

  fn get_subscription<'a>(&'a self) -> Option<Subscription<'a>> {
    match self.get_header("subscription"){
      Some(h) => Some(Subscription(h.get_value())),
      None => None
    }
  }

  fn get_transaction<'a>(&'a self) -> Option<Transaction<'a>> {
    match self.get_header("transaction"){
      Some(h) => Some(Transaction(h.get_value())),
      None => None
    }
  }

  fn get_version(&self) -> Option<Version> {
    let version = match self.get_header("version"){
      Some(h) => h.get_value(),
      None => return None
    };
    match (version).as_slice() {
      "1.0" => Some(Version(StompVersion::Stomp_v1_0)), // TODO: Impl FromStr for StompVersion
      "1.1" => Some(Version(StompVersion::Stomp_v1_1)),
      "1.2" => Some(Version(StompVersion::Stomp_v1_2)),
      _ => None
    }
  }

  fn get_content_length(&self) -> Option<ContentLength> {
    let length = match self.get_header("content-length") {
      Some(h) => h.get_value(),
      None => return None
    };
    match length.parse::<u32>() {
      Some(l) => Some(ContentLength(l)),
      None => None
    }
  }


}

#[macro_export]
macro_rules! header_list [
  ($($header: expr), *) => ({
    let mut header_list = HeaderList::new();
    $(header_list.push($header);)*
    header_list
  });
  ($($key:expr => $value: expr), *) => ({
    let mut header_list = HeaderList::new();
    $(header_list.push(Header::encode_key_value($key, $value));)*
    header_list
  })

];
