// Non-camel case types are used for Stomp Protocol version enum variants
#![allow(non_camel_case_types)]

use std::slice::Items;

// Ideally this would be a simple typedef. However:
// See Rust bug #11047: https://github.com/mozilla/rust/issues/11047
// Cannot call static methods (`with_capacity`) on type aliases (`HeaderList`)
pub struct HeaderList {
  pub headers: Vec<Header>
}

impl HeaderList {
  pub fn new(capacity: uint) -> HeaderList {
    HeaderList {
      headers: Vec::with_capacity(capacity)
    }
  }

  pub fn push(&mut self, header: Header) {
    self.headers.push(header);
  }

  pub fn iter<'a>(&'a self) -> Items<'a, Header> {
    self.headers.iter()
  }

  pub fn get_content_length(& self) -> Option<ContentLength> {
    let length = match self.get_header("content-length") {
      Some(&Header(_, ref v)) => v,
      None => return None
    };
    match from_str::<uint>(*length) {
      Some(l) => Some(ContentLength(l)),
      None => None
    }
  }


}


pub struct Header(~str, ~str);

impl Header {
  pub fn get_key<'a>(&'a self) -> &'a ~str {
    match self {
      &Header(ref k, _) => k
    }
  }
  pub fn get_value<'a>(&'a self) -> &'a ~str {
    match self {
      &Header(_, ref v) => v
    }
  }
  pub fn from_str(string : &str) -> Option<Header> {
    let parts : ~[&str] = string.split(':').collect();
    match parts.as_slice() {
      [k, v] => Some(Header(k.to_owned(), v.to_owned())),
      _ => None
    }
  }
}

// Headers in the Spec
pub struct AcceptVersion(~[StompVersion]);
pub struct ContentLength(pub uint);
pub struct Custom(Header);
pub struct Destination(~str);
pub struct HeartBeat(uint, uint);
pub struct Host(~str);
pub struct Id(~str);
pub struct Login(~str);
pub struct Passcode(~str);
pub struct Receipt(~str);
pub struct ReceiptId(~str);
pub struct Server(~str);
pub struct Session(~str);
pub struct Subscription(~str);
pub struct Transaction(~str);
pub struct Version(StompVersion);

pub enum StompVersion {
  Stomp_v1_0,
  Stomp_v1_1,
  Stomp_v1_2,
}

pub enum AckMode {
  Auto,
  Client,
  ClientIndividual
}


trait StompHeaderSet {
  fn get_header<'a>(&'a self, key: &str) -> Option<&'a Header>;
  fn get_accept_version<'a>(&'a self) -> Option<~[StompVersion]>;
  fn get_destination(&self) -> Option<Destination>;
  fn get_heart_beat(&self) -> Option<HeartBeat>;
  fn get_host(&self) -> Option<Host>;
  fn get_id(&self) -> Option<Id>;
  fn get_login(&self) -> Option<Login>;
  fn get_passcode(&self) -> Option<Passcode>;
  fn get_receipt(&self) -> Option<Receipt>;
  fn get_receipt_id(&self) -> Option<ReceiptId>;
  fn get_server(&self) -> Option<Server>;
  fn get_session(&self) -> Option<Session>;
  fn get_subscription(&self) -> Option<Subscription>;
  fn get_transaction(&self) -> Option<Transaction>;
  fn get_version(&self) -> Option<Version>;
}

impl StompHeaderSet for HeaderList {
  
  fn get_header<'a>(&'a self, key: &str) -> Option<&'a Header>{
    self.headers.iter().find(|header| 
      match **header {
        Header(ref k, _) if (*k).as_slice() == key => true, 
        _ => false
      }
    )
  }

  fn get_accept_version<'a>(&'a self) -> Option<~[StompVersion]> {
    let versions : &~str = match self.get_header("accept-version") {
      Some(&Header(_, ref v)) => v,
      None => return None
    };
    let versions: ~[StompVersion] = versions.split(',').filter_map(|v| match v.trim() {
      "1.0" => Some(Stomp_v1_0),
      "1.1" => Some(Stomp_v1_1),
      "1.2" => Some(Stomp_v1_2),
      _ => None
    }).collect();
    Some(versions)
  }
  fn get_destination(&self) -> Option<Destination> {
    match self.get_header("destination") {
      Some(&Header(_, ref v)) => Some(Destination(v.to_owned())),
      None => return None
    }
  }

  fn get_heart_beat(&self) -> Option<HeartBeat> {
    let spec = match self.get_header("heart-beat") {
      Some(&Header(_, ref v)) => v, 
      None => return None
    };
    let spec_list: ~[uint] = spec.split(',').filter_map(|str_val| from_str::<uint>(str_val)).collect();
    match spec_list.as_slice() {
      [x, y] => Some(HeartBeat(x, y)),
      _ => None
    }
  }

  fn get_host(&self) -> Option<Host> {
    match self.get_header("host") {
      Some(&Header(_, ref v)) => Some(Host(v.to_owned())),
      None => None
    }
  }
  
  fn get_id(&self) -> Option<Id> {
    match self.get_header("id") {
      Some(&Header(_, ref v)) => Some(Id(v.to_owned())),
      None => None
    }
  }

  fn get_login(&self) -> Option<Login> {
    match self.get_header("login"){
      Some(&Header(_, ref v)) => Some(Login(v.to_owned())),
      None => None
    }
  }

  fn get_passcode(&self) -> Option<Passcode> {
    match self.get_header("passcode"){
      Some(&Header(_, ref v)) => Some(Passcode(v.to_owned())),
      None => None
    }
  }

  fn get_receipt(&self) -> Option<Receipt> {
    match self.get_header("receipt"){
      Some(&Header(_, ref v)) => Some(Receipt(v.to_owned())),
      None => None
    }
  }

  fn get_receipt_id(&self) -> Option<ReceiptId> {
    match self.get_header("receipt-id"){
      Some(&Header(_, ref v)) => Some(ReceiptId(v.to_owned())),
      None => None
    }
  }

  fn get_server(&self) -> Option<Server> {
    match self.get_header("server"){
      Some(&Header(_, ref v)) => Some(Server(v.to_owned())),
      None => None
    }
  }

  fn get_session(&self) -> Option<Session> {
    match self.get_header("session"){
      Some(&Header(_, ref v)) => Some(Session(v.to_owned())),
      None => None
    }
  }

  fn get_subscription(&self) -> Option<Subscription> {
    match self.get_header("subscription"){
      Some(&Header(_, ref v)) => Some(Subscription(v.to_owned())),
      None => None
    }
  }

  fn get_transaction(&self) -> Option<Transaction> {
    match self.get_header("transaction"){
      Some(&Header(_, ref v)) => Some(Transaction(v.to_owned())),
      None => None
    }
  }

  fn get_version(&self) -> Option<Version> {
    let version = match self.get_header("version"){
      Some(&Header(_, ref v)) => v,
      None => return None
    };
    match (*version).as_slice() {
      "1.0" => Some(Version(Stomp_v1_0)), // TODO: Impl FromStr for StompVersion
      "1.1" => Some(Version(Stomp_v1_1)),
      "1.2" => Some(Version(Stomp_v1_2)),
      _ => None
    }
  }

}

