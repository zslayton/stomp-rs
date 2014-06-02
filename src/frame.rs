use headers::HeaderList;
use headers::Header;
use headers::ContentLength;
use headers::StompHeaderSet;
use std::io::IoResult;
use std::io::IoError;
use std::io::InvalidInput;
use std::io::BufferedReader;
use std::str::from_utf8;

pub struct Frame {
  pub command : String,
  pub headers : HeaderList, 
  pub body : Vec<u8>
}

impl Frame {
  // TODO: Rewrite properly with String to reduce allocation / simplify
  pub fn to_str(&self) -> String {
    let mut header_str = "".to_string(); 
    for header in self.headers.iter() {
      header_str = format!("{}{}:{}\n", header_str, header.get_key(), header.get_value());
    }
    let body_str : &str = match from_utf8(self.body.as_slice()) {
      Some(ref s) => *s,
      None => "<Binary content>"
    };

    format!("command: {}\nheaders: {}\nbody: {}", self.command, header_str, body_str)
  }

  pub fn write<T: Writer>(&self, stream: &mut T) -> IoResult<()> {
    try!(stream.write_str(self.command.as_slice()));
    for header in self.headers.iter() {
      try!(stream.write_str(header.get_key()));
      try!(stream.write_str(":"));
      try!(stream.write_str(header.get_value()));
    }
    let result = try!(stream.write(self.body.as_slice()));
    Ok(result)
  }

  pub fn read<R: Reader>(stream: &mut BufferedReader<R>) -> IoResult<Frame> {
    let mut line : String;
    loop {
      line = try!(stream.read_line()).as_slice().trim_right_chars(&['\r', '\n']).to_string();
      if line.len() > 0 {
        break;
      }
    }
    let command : String = line;

    let mut header_list : HeaderList = HeaderList::new(3);
    loop {
      line = try!(stream.read_line()).as_slice().trim_right_chars(&['\r', '\n']).to_string();
      if line.len() == 0 {
        break;
      }
      let header = Header::from_str(line.as_slice());
      match header {
        Some(h) => header_list.push(h),
        None => return Err(IoError{kind: InvalidInput, desc: "Invalid header encountered.", detail: Some(line)})
      }
    }

    let body: Vec<u8>; 
    let content_length = header_list.get_content_length();
    match content_length {
      Some(ContentLength(num_bytes)) => {
        body = try!(stream.read_exact(num_bytes));
      },
      None => {
        body = try!(stream.read_until(0 as u8));
      }
    }
    Ok(Frame{command: command, headers: header_list, body:body}) 
  }
}
