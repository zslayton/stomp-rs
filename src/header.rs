// Non-camel case types are used for Stomp Protocol version enum variants
#![macro_use]
#![allow(non_camel_case_types)]
use std::slice::Iter;
use unicode_segmentation::UnicodeSegmentation;

// Ideally this would be a simple typedef. However:
// See Rust bug #11047: https://github.com/mozilla/rust/issues/11047
// Cannot call static methods (`with_capacity`) on type aliases (`HeaderList`)
#[derive(Clone, Debug)]
pub struct HeaderList {
    pub headers: Vec<Header>,
}

impl HeaderList {
    pub fn new() -> HeaderList {
        HeaderList::with_capacity(0)
    }
    pub fn with_capacity(capacity: usize) -> HeaderList {
        HeaderList {
            headers: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, header: Header) {
        self.headers.push(header);
    }

    pub fn pop(&mut self) -> Option<Header> {
        self.headers.pop()
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, Header> {
        self.headers.iter()
    }

    pub fn drain<F>(&mut self, mut sink: F)
    where
        F: FnMut(Header),
    {
        while let Some(header) = self.headers.pop() {
            sink(header);
        }
    }

    pub fn concat(&mut self, other_list: &mut HeaderList) {
        other_list.headers.reverse();
        while let Some(header) = other_list.pop() {
            self.headers.push(header);
        }
    }

    pub fn retain<F>(&mut self, test: F)
    where
        F: Fn(&Header) -> bool,
    {
        self.headers.retain(test)
    }
}

pub struct SuppressedHeader<'a>(pub &'a str);
pub struct ContentType<'a>(pub &'a str);
#[derive(Clone, Debug)]
pub struct Header(pub String, pub String);

impl Header {
    pub fn new(key: &str, value: &str) -> Header {
        Header(Self::encode_value(key), Self::encode_value(value))
    }

    pub fn new_raw<T: Into<String>, U: Into<String>>(key: T, value: U) -> Header {
        Header(key.into(), value.into())
    }

    pub fn get_raw(&self) -> String {
        format!("{}:{}", self.0, self.1)
    }

    pub fn encode_value(value: &str) -> String {
        let mut encoded = String::new(); //self.strings.detached();
        for grapheme in UnicodeSegmentation::graphemes(value, true) {
            match grapheme {
                "\\" => encoded.push_str(r"\\"), // Order is significant
                "\r" => encoded.push_str(r"\r"),
                "\n" => encoded.push_str(r"\n"),
                ":" => encoded.push_str(r"\c"),
                g => encoded.push_str(g),
            }
        }
        encoded
    }

    pub fn get_key<'a>(&'a self) -> &'a str {
        &self.0
    }

    pub fn get_value<'a>(&'a self) -> &'a str {
        &self.1
    }
}

// Headers in the Spec
#[derive(Clone)]
pub struct AcceptVersion(pub Vec<StompVersion>);
pub struct Ack<'a>(pub &'a str);
#[derive(Clone, Copy)]
pub struct ContentLength(pub u32);
pub struct Custom(pub Header);
pub struct Destination<'a>(pub &'a str);
#[derive(Clone, Copy)]
pub struct HeartBeat(pub u32, pub u32);
pub struct Host<'a>(pub &'a str);
pub struct Id<'a>(pub &'a str);
pub struct Login<'a>(pub &'a str);
pub struct MessageId<'a>(pub &'a str);
pub struct Passcode<'a>(pub &'a str);
pub struct Receipt<'a>(pub &'a str);
pub struct ReceiptId<'a>(pub &'a str);
pub struct Server<'a>(pub &'a str);
pub struct Session<'a>(pub &'a str);
pub struct Subscription<'a>(pub &'a str);
pub struct Transaction<'a>(pub &'a str);
#[derive(Clone, Copy)]
pub struct Version(pub StompVersion);

#[derive(Clone, Copy)]
pub enum StompVersion {
    Stomp_v1_0,
    Stomp_v1_1,
    Stomp_v1_2,
}

impl HeaderList {
    pub fn get_header<'a>(&'a self, key: &str) -> Option<&'a Header> {
        self.headers.iter().find(|header| match **header {
            ref h if h.get_key() == key => true,
            _ => false,
        })
    }

    pub fn get_accept_version(&self) -> Option<Vec<StompVersion>> {
        let versions: &str = match self.get_header("accept-version") {
            Some(h) => h.get_value(),
            None => return None,
        };
        let versions: Vec<StompVersion> = versions
            .split(',')
            .filter_map(|v| match v.trim() {
                "1.0" => Some(StompVersion::Stomp_v1_0),
                "1.1" => Some(StompVersion::Stomp_v1_1),
                "1.2" => Some(StompVersion::Stomp_v1_2),
                _ => None,
            })
            .collect();
        Some(versions)
    }

    pub fn get_ack<'a>(&'a self) -> Option<Ack<'a>> {
        match self.get_header("ack") {
            Some(h) => Some(Ack(h.get_value())),
            None => None,
        }
    }

    pub fn get_destination<'a>(&'a self) -> Option<Destination<'a>> {
        match self.get_header("destination") {
            Some(h) => Some(Destination(h.get_value())),
            None => return None,
        }
    }

    pub fn get_heart_beat(&self) -> Option<HeartBeat> {
        let spec = match self.get_header("heart-beat") {
            Some(h) => h.get_value(),
            None => return None,
        };
        let spec_list: Vec<u32> = spec
            .split(',')
            .filter_map(|str_val| str_val.parse::<u32>().ok())
            .collect();

        if spec_list.len() != 2 {
            return None;
        }
        Some(HeartBeat(spec_list[0], spec_list[1]))
    }

    pub fn get_host<'a>(&'a self) -> Option<Host<'a>> {
        match self.get_header("host") {
            Some(h) => Some(Host(h.get_value())),
            None => None,
        }
    }

    pub fn get_id<'a>(&'a self) -> Option<Id<'a>> {
        match self.get_header("id") {
            Some(h) => Some(Id(h.get_value())),
            None => None,
        }
    }

    pub fn get_login<'a>(&'a self) -> Option<Login<'a>> {
        match self.get_header("login") {
            Some(h) => Some(Login(h.get_value())),
            None => None,
        }
    }

    pub fn get_message_id<'a>(&'a self) -> Option<MessageId<'a>> {
        match self.get_header("message-id") {
            Some(h) => Some(MessageId(h.get_value())),
            None => None,
        }
    }

    pub fn get_passcode<'a>(&'a self) -> Option<Passcode<'a>> {
        match self.get_header("passcode") {
            Some(h) => Some(Passcode(h.get_value())),
            None => None,
        }
    }

    pub fn get_receipt<'a>(&'a self) -> Option<Receipt<'a>> {
        match self.get_header("receipt") {
            Some(h) => Some(Receipt(h.get_value())),
            None => None,
        }
    }

    pub fn get_receipt_id<'a>(&'a self) -> Option<ReceiptId<'a>> {
        match self.get_header("receipt-id") {
            Some(h) => Some(ReceiptId(h.get_value())),
            None => None,
        }
    }

    pub fn get_server<'a>(&'a self) -> Option<Server<'a>> {
        match self.get_header("server") {
            Some(h) => Some(Server(h.get_value())),
            None => None,
        }
    }

    pub fn get_session<'a>(&'a self) -> Option<Session<'a>> {
        match self.get_header("session") {
            Some(h) => Some(Session(h.get_value())),
            None => None,
        }
    }

    pub fn get_subscription<'a>(&'a self) -> Option<Subscription<'a>> {
        match self.get_header("subscription") {
            Some(h) => Some(Subscription(h.get_value())),
            None => None,
        }
    }

    pub fn get_transaction<'a>(&'a self) -> Option<Transaction<'a>> {
        match self.get_header("transaction") {
            Some(h) => Some(Transaction(h.get_value())),
            None => None,
        }
    }

    pub fn get_version(&self) -> Option<Version> {
        let version = match self.get_header("version") {
            Some(h) => h.get_value(),
            None => return None,
        };
        match (version).as_ref() {
            "1.0" => Some(Version(StompVersion::Stomp_v1_0)), // TODO: Impl FromStr for StompVersion
            "1.1" => Some(Version(StompVersion::Stomp_v1_1)),
            "1.2" => Some(Version(StompVersion::Stomp_v1_2)),
            _ => None,
        }
    }

    pub fn get_content_length(&self) -> Option<ContentLength> {
        let length = match self.get_header("content-length") {
            Some(h) => h.get_value(),
            None => return None,
        };
        match length.parse::<u32>().ok() {
            Some(l) => Some(ContentLength(l)),
            None => None,
        }
    }
}

#[macro_export]
macro_rules! header_list [
  ($($header: expr), *) => ({
    let header_list = HeaderList::new();
    $(header_list.push($header);)*
    header_list
  });
  ($($key:expr => $value: expr), *) => ({
    let mut header_list = HeaderList::new();
    $(header_list.push(Header::new($key, $value));)*
    header_list
  })

];

#[test]
fn encode_return_carriage() {
    let unencoded = "Hello\rWorld";
    let encoded = r"Hello\rWorld";
    assert!(encoded == Header::encode_value(unencoded));
}

#[test]
fn encode_newline() {
    let unencoded = "Hello\nWorld";
    let encoded = r"Hello\nWorld";
    assert!(encoded == Header::encode_value(unencoded));
}

#[test]
fn encode_colon() {
    let unencoded = "Hello:World";
    let encoded = r"Hello\cWorld";
    assert!(encoded == Header::encode_value(unencoded));
}

#[test]
fn encode_slash() {
    let unencoded = r"Hello\World";
    let encoded = r"Hello\\World";
    assert!(encoded == Header::encode_value(unencoded));
}
