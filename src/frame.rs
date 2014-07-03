use headers::HeaderList;
use headers::Header;
use headers::ContentLength;
use headers::StompHeaderSet;
use std::io::IoResult;
use std::io::IoError;
use std::io::InvalidInput;
use std::io::BufferedReader;
use std::str::from_utf8;

use std::fmt::Show;
use std::fmt::Formatter;
use std::fmt::Result;

pub struct Frame {
  pub command : String,
  pub headers : HeaderList, 
  pub body : Vec<u8>
}

impl Show for Frame {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.to_str())
    }
}

impl Frame {

  pub fn character_count(&self) -> uint {
     let mut space_required = 0u;
    // Add one to space calculations to make room for '\n'
    space_required += self.command.len() + 1;
    space_required += self.headers.iter()
                      .fold(0, |length, header| 
                          length + header.get_raw().len()+1
                      );
    space_required += 1; // Newline at end of headers
    space_required += self.body.len();
    space_required
  }

  pub fn to_str(&self) -> String {
    let space_required = self.character_count();
    println!("Required Space: {}", space_required);
    let mut frame_string = String::with_capacity(space_required);
    frame_string = frame_string.append(self.command.as_slice());
    frame_string = frame_string.append("\n");
    for header in self.headers.iter() {
      frame_string = frame_string.append(header.get_raw());
      frame_string = frame_string.append("\n");
    }
    frame_string = frame_string.append("\n");
    let body_string : &str = match from_utf8(self.body.as_slice()) {
      Some(ref s) => *s,
      None => "<Binary content>" // Space is wasted in this case. Could shrink to fit?
    };
    frame_string = frame_string.append(body_string);
    println!("Final string length: {}", frame_string.len());
    frame_string
  }

  pub fn write<T: Writer>(&self, stream: &mut T) -> IoResult<()> {
    println!("Sending command...");
    try!(stream.write_str(self.command.as_slice()));
    try!(stream.write_str("\n"));
    println!("Sending headers...");
    for header in self.headers.iter() {
      try!(stream.write_str(header.get_raw()));
      try!(stream.write_str("\n"));
    }
    try!(stream.write_str("\n"));
    println!("Sending body...");
    try!(stream.write(self.body.as_slice()));
    try!(stream.write(&[0]));
    println!("write() complete.");
    Ok(())
  }

  fn chomp_line(mut line: String) -> String {
    // This may be suboptimal, but fine for now
    let chomped_length : uint;
    {
      let trimmed_line : &str = line.as_slice().trim_right_chars(&['\r', '\n']);
      chomped_length = trimmed_line.len();
    }
    line.truncate(chomped_length);
    line
  }

  pub fn read<R: Reader>(stream: &mut BufferedReader<R>) -> IoResult<Frame> {
    let mut line : String;
    // Consume any number of empty preceding lines
    loop {
      line = Frame::chomp_line(try!(stream.read_line()));
      if line.len() > 0 {
        break;
      }
    }
    let command : String = line;

    let mut header_list : HeaderList = HeaderList::with_capacity(3);
    loop {
      line = Frame::chomp_line(try!(stream.read_line()));
      if line.len() == 0 { // Empty line, no more headers
        break;
      }
      let header = Header::from_str(line.as_slice());
      match header {
        Some(h) => header_list.push(h),
        None => return Err(IoError{kind: InvalidInput, desc: "Invalid header encountered.", detail: Some(line.to_string())})
      }
    }

    let body: Vec<u8>; 
    let content_length = header_list.get_content_length();
    match content_length {
      Some(ContentLength(num_bytes)) => {
        body = try!(stream.read_exact(num_bytes));
        let _ = try!(stream.read_exact(1)); // Toss aside trailing null octet
      },
      None => {
        body = try!(stream.read_until(0 as u8));
      }
    }
    Ok(Frame{command: command, headers: header_list, body:body}) 
  }

  pub fn subscribe(topic: &str, subscription_id: uint) -> Frame {
    let mut header_list : HeaderList = HeaderList::with_capacity(3);
    let subscription_id_str = format!("stomp-rs/{}", subscription_id);
    header_list.push(Header::from_key_value("destination", topic));
    header_list.push(Header::from_key_value("id", subscription_id_str.as_slice()));
    header_list.push(Header::from_key_value("ack", "auto"));
    let subscribe_frame = Frame {
      command : "SUBSCRIBE".to_string(),
      headers : header_list,
      body : Vec::new()
    };
    subscribe_frame
  }

  pub fn send(topic: &str, body: &[u8]) -> Frame {
    let mut header_list : HeaderList = HeaderList::with_capacity(3);
    header_list.push(Header::from_key_value("destination", topic));
    header_list.push(Header::from_key_value("content-length", body.len().to_str().as_slice()));
    header_list.push(Header::from_key_value("content-type","text/plain"));
    let send_frame = Frame {
      command : "SEND".to_string(),
      headers : header_list,
      body : Vec::from_slice(body)
    };
    send_frame
  }
}
