use header::HeaderList;
use header::Header;
use subscription::AckMode;
use std::str::from_utf8;
use std::fmt;
use std::fmt::Formatter;
use bytes::BytesMut;

#[derive(Copy, Clone, Debug)]
pub enum Command {
    Send,
    Subscribe,
    Unsubscribe,
    Begin,
    Commit,
    Abort,
    Ack,
    Nack,
    Disconnect,
    Connect,
    Stomp,
    Connected,
    Message,
    Receipt,
    Error
}
impl Command {
    pub fn as_str(&self) -> &'static str {
        use self::Command::*;

        match *self {
            Send => "SEND",
            Subscribe => "SUBSCRIBE",
            Unsubscribe => "UNSUBSCRIBE",
            Begin => "BEGIN",
            Commit => "COMMIT",
            Abort => "ABORT",
            Ack => "ACK",
            Nack => "NACK",
            Disconnect => "DISCONNECT",
            Connect => "CONNECT",
            Stomp => "STOMP",
            Connected => "CONNECTED",
            Message => "MESSAGE",
            Receipt => "RECEIPT",
            Error => "ERROR",
        }
    }
}
impl fmt::Display for Command {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
pub trait ToFrameBody {
    fn to_frame_body<'a>(&'a self) -> &'a [u8];
}

impl<'b> ToFrameBody for &'b [u8] {
    fn to_frame_body<'a>(&'a self) -> &'a [u8] {
        self
    }
}

impl<'b> ToFrameBody for &'b str {
    fn to_frame_body<'a>(&'a self) -> &'a [u8] {
        self.as_bytes()
    }
}

impl ToFrameBody for String {
    fn to_frame_body<'a>(&'a self) -> &'a [u8] {
        let string: &str = self.as_ref();
        string.as_bytes()
    }
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub command: Command,
    pub headers: HeaderList,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub enum Transmission {
    HeartBeat,
    CompleteFrame(Frame),
}

impl Transmission {
    pub fn write(&self, out: &mut BytesMut) {
        match *self {
            Transmission::HeartBeat => out.extend("\n".as_bytes()),
            Transmission::CompleteFrame(ref frame) => frame.write(out),
        }
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl Frame {
    pub fn count_bytes(&self) -> usize {
        let mut space_required: usize = 0;
        // Add one to space calculations to make room for '\n'
        space_required += self.command.as_str().len() + 1;
        space_required += self.headers
            .iter()
            .fold(0, |length, header| length + header.get_raw().len() + 1);
        space_required += 1; // Newline at end of headers
        space_required += self.body.len();
        space_required
    }

    pub fn to_str(&self) -> String {
        let space_required = self.count_bytes();
        let mut frame_string = String::with_capacity(space_required); // Faster to just allocate?
        frame_string.push_str(self.command.as_str());
        frame_string.push_str("\n");
        for header in self.headers.iter() {
            frame_string.push_str(&header.get_raw());
            frame_string.push_str("\n");
        }
        frame_string.push_str("\n");
        let body_string: &str = match from_utf8(self.body.as_ref()) {
            Ok(ref s) => *s,
            Err(_) => "<Binary content>", // Space is wasted in this case. Could shrink to fit?
        };
        frame_string.push_str(body_string);
        frame_string
    }

    pub fn write(&self, out: &mut BytesMut) {
        debug!("Sending frame:\n{}", self.to_str());
        out.extend(self.command.as_str().as_bytes());
        out.extend("\n".as_bytes());

        for header in self.headers.iter() {
            out.extend(header.get_raw().as_bytes());
            out.extend("\n".as_bytes());
        }

        out.extend("\n".as_bytes());
        out.extend(&self.body);

        out.extend(&[0]);
        debug!("write() complete.");
    }

    pub fn connect(tx_heartbeat_ms: u32, rx_heartbeat_ms: u32) -> Frame {
        let heart_beat = format!("{},{}", tx_heartbeat_ms, rx_heartbeat_ms);
        let connect_frame = Frame {
            command: Command::Connect,
            headers: header_list![
                "accept-version" => "1.2",
                "heart-beat" => heart_beat.as_ref(),
                "content-length" => "0"
            ],
            body: Vec::new(),
        };
        connect_frame
    }

    pub fn disconnect() -> Frame {
        let disconnect_frame = Frame {
            command: Command::Disconnect,
            headers: header_list![
                "receipt" => "msg/disconnect"
            ],
            body: Vec::new(),
        };
        disconnect_frame
    }


    pub fn subscribe(subscription_id: &str, destination: &str, ack_mode: AckMode) -> Frame {
        let subscribe_frame = Frame {
            command: Command::Subscribe,
            headers: header_list![
                "destination" => destination,
                "id" => subscription_id,
                "ack" => ack_mode.as_text()
            ],
            body: Vec::new(),
        };
        subscribe_frame
    }

    pub fn unsubscribe(subscription_id: &str) -> Frame {
        let unsubscribe_frame = Frame {
            command: Command::Unsubscribe,
            headers: header_list![
                "id" => subscription_id
            ],
            body: Vec::new(),
        };
        unsubscribe_frame
    }

    pub fn ack(ack_id: &str) -> Frame {
        let ack_frame = Frame {
            command: Command::Ack,
            headers: header_list![
                "id" => ack_id
            ],
            body: Vec::new(),
        };
        ack_frame
    }

    pub fn nack(message_id: &str) -> Frame {
        let nack_frame = Frame {
            command: Command::Nack,
            headers: header_list![
                "id" => message_id
            ],
            body: Vec::new(),
        };
        nack_frame
    }

    pub fn send(destination: &str, body: &[u8]) -> Frame {
        let send_frame = Frame {
            command: Command::Send,
            headers: header_list![
                "destination" => destination,
                "content-length" => body.len().to_string().as_ref()
            ],
            body: body.into(),
        };
        send_frame
    }

    pub fn begin(transaction_id: &str) -> Frame {
        let begin_frame = Frame {
            command: Command::Begin,
            headers: header_list![
                "transaction" => transaction_id
            ],
            body: Vec::new(),
        };
        begin_frame
    }

    pub fn abort(transaction_id: &str) -> Frame {
        let abort_frame = Frame {
            command: Command::Abort,
            headers: header_list![
                "transaction" => transaction_id
            ],
            body: Vec::new(),
        };
        abort_frame
    }

    pub fn commit(transaction_id: &str) -> Frame {
        let commit_frame = Frame {
            command: Command::Commit,
            headers: header_list![
                "transaction" => transaction_id
            ],
            body: Vec::new(),
        };
        commit_frame
    }
}
