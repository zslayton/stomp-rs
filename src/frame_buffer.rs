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
use std::io::BufRead;
use std::str::from_utf8;
use std::fmt;
use std::fmt::Formatter;
use frame::{Frame, Transmission};

pub struct FrameBuffer {
  buffer: Vec<u8>,
  parse_state: ParseState
}

#[derive(Clone, Copy)]
struct ByteRange {
  pub start: u32,
  pub end : u32
}

struct ParseState {
  offset: u32,
  command_range: Option<ByteRange>,
  header_ranges: Vec<ByteRange>,
  body_range: Option<ByteRange>,
  section: FrameSection,
}

impl ParseState {
  fn new() -> ParseState {
    ParseState {
      offset: 0,
      command_range: None,
      header_ranges: Vec::new(),
      body_range: None,
      section: FrameSection::Command
    }
  }
}

enum FrameSection {
  Command,
  Headers,
  Body
}

enum ReadCommandResult {
  HeartBeat,
  Command(ByteRange),
  Incomplete
}

enum ReadHeaderResult {
  Header(ByteRange),
  EndOfHeaders,
  Incomplete
}

enum ReadBodyResult {
  Body(ByteRange),
  Incomplete
}

impl FrameBuffer {
  pub fn new() -> FrameBuffer {
    FrameBuffer {
      buffer: Vec::with_capacity(512), 
      parse_state: ParseState::new()
    }
  }

  pub fn append(&mut self, bytes: &[u8]) { // TODO: Return result?
    self.buffer.push_all(bytes);
  }

  pub fn read_transmission(&mut self) -> Option<Transmission> {
    match self.parse_state.section {
      FrameSection::Command => self.resume_parsing_at_command(), 
      FrameSection::Headers => self.resume_parsing_at_headers(),
      FrameSection::Body    => self.resume_parsing_at_body()
    }
  }

  fn resume_parsing_at_command(&mut self) -> Option<Transmission> {
    debug!("Parsing command.");
    match self.read_command() {
      ReadCommandResult::HeartBeat => Some(Transmission::HeartBeat),
      ReadCommandResult::Command(byte_range) => {
        self.parse_state.command_range = Some(byte_range);
        self.parse_state.section = FrameSection::Headers;
        self.resume_parsing_at_headers()
      },
      ReadCommandResult::Incomplete => None
    }
  }

  fn resume_parsing_at_headers(&mut self) -> Option<Transmission> {
    debug!("Parsing headers.");
    match self.read_header() {
      ReadHeaderResult::Header(byte_range) => {
        self.parse_state.header_ranges.push(byte_range);
        self.resume_parsing_at_headers()
      },
      ReadHeaderResult::EndOfHeaders => {
        self.parse_state.section = FrameSection::Body;
        self.resume_parsing_at_body()
      },
      ReadHeaderResult::Incomplete => None
    }
  }

  fn resume_parsing_at_body(&mut self) -> Option<Transmission> {
    debug!("Parsing body.");
    match self.read_body() {
      ReadBodyResult::Body(byte_range) => {
        self.parse_state.body_range = Some(byte_range);
        let frame = Frame {
          command: self.create_command_string(),
          headers: self.create_header_list(),
          body: self.create_body()
        };
        self.reset_parse_state();
        Some(Transmission::CompleteFrame(frame))
      },
      ReadBodyResult::Incomplete => None
    }
  }

  fn reset_parse_state(&mut self) {
    self.buffer.clear();
    self.parse_state.offset = 0;
    self.parse_state.command_range = None;
    self.parse_state.header_ranges.clear();
    self.parse_state.body_range = None;
    self.parse_state.section = FrameSection::Command;
  }

  fn create_command_string(&mut self) -> String {
    let buffer : &[u8] = self.buffer.as_ref();
    let command_range = self.parse_state.command_range.expect("No command range was found.");
    let start = command_range.start;
    let end = command_range.end;
    let command_slice = &buffer[start as usize..end as usize];
    let command_str : &str = match from_utf8(&command_slice) {
      Ok(command_str) => command_str,
      _ =>panic!("Command was not utf8.")
    };
    return command_str.to_string();
  }

  fn create_header_list(&mut self) -> HeaderList {
    let buffer : &[u8] = self.buffer.as_ref();
    let mut header_list = header_list![];
    for header_range in &self.parse_state.header_ranges {
      let start = header_range.start;
      let end = header_range.end;
      let header_slice = &buffer[start as usize..end as usize];
      let header_str : &str = match from_utf8(&header_slice) {
        Ok(header_str) => header_str,
        _ =>panic!("Header was not utf8.")
      };
      let header = Header::decode_string(header_str);
      match header {
        Some(header) => header_list.push(header),
        None => panic!("Header was not decodable.")
      } 
    }
    header_list    
  }

  fn create_body(&mut self) -> Vec<u8> {
    let buffer : &[u8] = self.buffer.as_ref();
    let body_range = self.parse_state.body_range.expect("No body range was found.");
    let start = body_range.start;
    let end = body_range.end;
    let body_slice = &buffer[start as usize ..end as usize];
    return Vec::from(body_slice);
  }

  fn read_command(&self) -> ReadCommandResult {
    let offset = self.parse_state.offset;
    match self.find_next(offset, '\n' as u8) {
      Some(index) => {
        self.parse_state.offset += index - offset;
        if index == offset {
          ReadCommandResult::HeartBeat(ByteRange{start: offset, end: index})
        } else {
          ReadCommandResult::Command(ByteRange{start: offset, end: index})
        }
      }
      None => ReadCommandResult::Incomplete
    }
  }

  fn read_header(&self) -> ReadHeaderResult {
    let offset = self.parse_state.offset;
    match self.find_next(offset, '\n' as u8) {
      Some(index) => {
        self.parse_state.offset += index - offset;
        ReadHeaderResult::Header(ByteRange{start: offset, end: index})
      },
      None => ReadHeaderResult::Incomplete
    }
  }
  
  // Need to read both by Content-Length and by null byte
  fn read_body(&self) -> ReadBodyResult {
    ReadBodyResult::Incomplete
  }

  fn find_next(&self, offset: u32, needle: u8) -> Option<u32> {
    // What's the len() of the underlying slice? capacity or size?
    let mut step = 0u32;
    for byte in &self.buffer[offset as usize..] {
      if *byte == needle {
        return Some(offset + step);
      }
      step += 1;
    }
    None
  }
}
