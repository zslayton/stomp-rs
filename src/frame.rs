use header::HeaderList;
use header::Header;
use header::ContentLength;
use header::StompHeaderSet;
use subscription::AckMode;
use std::io::Result;
use std::io::Error;
use std::io::ErrorKind::InvalidInput;
use std::io::Write;
use std::io::Read;
use std::io::BufWriter;
use std::io::BufRead;
use std::str::from_utf8;
use std::slice::AsSlice;
use std::str::Str;
use std::fmt;
use std::fmt::Formatter;

pub trait ToFrameBody {
  fn to_frame_body<'a>(&'a self) -> &'a [u8];
}

impl <T> ToFrameBody for T where T: AsSlice<u8> {
  fn to_frame_body<'a>(&'a self) -> &'a [u8] {
    self.as_slice()
  } 
}

impl ToFrameBody for &'static str { 
  fn to_frame_body<'a>(&'a self) -> &'a [u8] {
    self.as_slice().as_bytes()
  } 
}

impl ToFrameBody for String { 
  fn to_frame_body<'a>(&'a self) -> &'a [u8] {
    self.as_slice().as_bytes()
  } 
}

#[derive(Clone)]
pub struct Frame {
  pub command : String,
  pub headers : HeaderList, 
  pub body : Vec<u8>
}

pub enum Transmission {
  HeartBeat,
  CompleteFrame(Frame)
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl Frame {

  pub fn count_bytes(&self) -> usize {
     let mut space_required : usize = 0;
    // Add one to space calculations to make room for '\n'
    space_required += self.command.len() + 1;
    space_required += self.headers.iter()
                      .fold(0, |length, header| 
                          length + header.get_raw().len() +1
                      );
    space_required += 1; // Newline at end of headers
    space_required += self.body.len();
    space_required
  }

  pub fn to_str(&self) -> String {
    let space_required = self.count_bytes();
    let mut frame_string = String::with_capacity(space_required);
    frame_string.push_str(self.command.as_slice());
    frame_string.push_str("\n");
    for header in self.headers.iter() {
      frame_string.push_str(header.get_raw());
      frame_string.push_str("\n");
    }
    frame_string.push_str("\n");
    let body_string : &str = match from_utf8(self.body.as_slice()) {
      Ok(ref s) => *s,
      Err(_) => "<Binary content>" // Space is wasted in this case. Could shrink to fit?
    };
    frame_string.push_str(body_string);
    frame_string
  }

  pub fn write<W: Write>(&self, stream: &mut BufWriter<W>) -> Result<()> {
    debug!("Sending frame:\n{}", self.to_str());
    try!(stream.write(self.command.as_slice().as_bytes()));
    try!(stream.write("\n".as_bytes()));
    for header in self.headers.iter() {
      try!(stream.write(header.get_raw().as_bytes()));
      try!(stream.write("\n".as_bytes()));
    }
    try!(stream.write("\n".as_bytes()));
    try!(stream.write_all(self.body.as_slice()));
    try!(stream.write_all(&[0]));
    try!(stream.flush());
    debug!("write() complete.");
    Ok(())
  }

  fn chomp_line(mut line: String) -> String {
    // This may be suboptimal, but fine for now
    let chomped_length : usize;
    {
      let chars_to_remove : &[char] = &['\r', '\n'];
      let trimmed_line : &str = line.as_slice().trim_right_matches(chars_to_remove);
      chomped_length = trimmed_line.len();
    }
    line.truncate(chomped_length);
    line
  }

  pub fn read<B: BufRead>(stream: &mut B) -> Result<Transmission> {
    let mut line : String = String::new();
    // Empty lines are interpreted as heartbeats
    try!(stream.read_line(&mut line));
    line = Frame::chomp_line(line);
    if line.len() == 0 {
      return Ok(Transmission::HeartBeat);
    }

    let command : String = line;

    let mut header_list : HeaderList = HeaderList::with_capacity(6);
    loop {
      line = String::new();
      try!(stream.read_line(&mut line));
      line = Frame::chomp_line(line);
      if line.len() == 0 { // Empty line, no more headers
        break;
      }
      let header = Header::decode_string(line.as_slice());
      match header {
        Some(h) => header_list.push(h),
        None => return Err(Error::new(InvalidInput, "Invalid header encountered.", Some(line.to_string())))
      }
    }

    let mut body: Vec<u8>; 
    let content_length = header_list.get_content_length();
    match content_length {
      Some(ContentLength(num_bytes)) => {
        body = try!(stream.take((num_bytes+1) as u64).bytes().collect()); // bytes + 1 for trailing null octet
        body.pop(); // Remove trailing octet
      },
      None => {
        body = Vec::new();
        let _ = try!(stream.read_until(0 as u8, &mut body));
      }
    }
    Ok(Transmission::CompleteFrame(Frame{command: command, headers: header_list, body:body}))
  }

  pub fn connect(tx_heartbeat_ms: u32, rx_heartbeat_ms: u32) -> Frame {
    let heart_beat = format!("{},{}", tx_heartbeat_ms, rx_heartbeat_ms);
    let connect_frame = Frame {
       command : "CONNECT".to_string(),
       headers : header_list![
         "accept-version" => "1.2",
         "heart-beat" => heart_beat.as_slice(),
         "content-length" => "0"
       ],
       body : Vec::new() 
    };
    connect_frame
  }

  pub fn disconnect() -> Frame {
    let disconnect_frame = Frame {
       command : "DISCONNECT".to_string(),
       headers : header_list![
         "receipt" => "msg/disconnect"
       ],
       body : Vec::new() 
    };
    disconnect_frame
  }


  pub fn subscribe(subscription_id: &str, topic: &str, ack_mode: AckMode) -> Frame {
    let subscribe_frame = Frame {
      command : "SUBSCRIBE".to_string(),
      headers : header_list![
        "destination" => topic,
        "id" => subscription_id,
        "ack" => ack_mode.as_text()
      ],
      body : Vec::new()
    };
    subscribe_frame
  }

  pub fn unsubscribe(subscription_id: &str) -> Frame {
    let unsubscribe_frame = Frame {
      command : "UNSUBSCRIBE".to_string(),
      headers : header_list![
        "id" => subscription_id
      ],
      body : Vec::new()
    };
    unsubscribe_frame
  }

  pub fn ack(ack_id: &str) -> Frame {
    let ack_frame = Frame {
      command : "ACK".to_string(),
      headers : header_list![
        "id" => ack_id
      ],
      body : Vec::new()
    };
    ack_frame
  }

  pub fn nack(message_id: &str) -> Frame {
    let nack_frame= Frame {
      command : "NACK".to_string(),
      headers : header_list![
        "id" => message_id
      ],
      body : Vec::new()
    };
    nack_frame
  }

  pub fn send(topic: &str, body: &[u8]) -> Frame {
    let send_frame = Frame {
      command : "SEND".to_string(),
      headers : header_list![
        "destination" => topic,
        "content-length" => body.len().to_string().as_slice()
      ],
      body : body.to_vec()
    };
    send_frame
  }

  pub fn begin(transaction_id: &str) -> Frame {
    let begin_frame = Frame {
      command : "BEGIN".to_string(),
      headers : header_list![
        "transaction" => transaction_id
      ],
      body : Vec::new()
    };
    begin_frame 
  }

  pub fn abort(transaction_id: &str) -> Frame {
    let abort_frame = Frame {
      command : "ABORT".to_string(),
      headers : header_list![
        "transaction" => transaction_id
      ],
      body : Vec::new()
    };
    abort_frame 
  }

  pub fn commit(transaction_id: &str) -> Frame {
    let commit_frame = Frame {
      command : "COMMIT".to_string(),
      headers : header_list![
        "transaction" => transaction_id
      ],
      body : Vec::new()
    };
    commit_frame 
  }
}
