use mai;
use mai::{DecodingResult, DecodedFrame, DecodingError, BytesWritten, BytesRead, EncodingResult,
          EncodingError};
#[macro_use]
use header;
use header::HeaderList;
use header::Header;
use header::ContentLength;
use header::StompHeaderSet;
use header::HeaderCodec;
use std::str::from_utf8;
use std::mem;
use frame::{Frame, Transmission};
use lifeguard::Pool;

const DEFAULT_STRING_POOL_SIZE: usize = 4;
const DEFAULT_STRING_POOL_MAX_SIZE: usize = 32;
const DEFAULT_HEADER_CODEC_STRING_POOL_SIZE: usize = 16;
const DEFAULT_HEADER_CODEC_STRING_POOL_MAX_SIZE: usize = 64;

pub struct Codec {
    // buffer: VecDeque<u8>,
    parse_state: ParseState,
    // string_pool: Pool<String>,
    header_codec: HeaderCodec,
}

struct ParseState {
    command: Option<String>,
    headers: HeaderList,
    section: FrameSection,
    offset: usize,
}

impl ParseState {
    fn new() -> ParseState {
        ParseState {
            command: None,
            headers: header_list![],
            section: FrameSection::Command,
            offset: 0,
        }
    }
}

enum FrameSection {
    Command,
    Headers,
    Body,
}

pub struct FrameBuffer<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> FrameBuffer<'a> {
    pub fn from(bytes: &'a [u8], offset: usize) -> FrameBuffer<'a> {
        FrameBuffer {
            bytes: bytes,
            offset: offset
        }
    }

    pub fn remaining(&self) -> &[u8] {
        &self.bytes[self.offset..]
    }

    pub fn consumed(&self) -> usize {
        self.offset
    }

    pub fn find_next(&self, needle: u8) -> Option<usize> {
        debug!("Searching for next {} starting from {}", needle, self.offset);
        self.remaining().iter().position(|byte| *byte == needle)
    }

    fn read_into_vec(&mut self, n: usize) -> Vec<u8> {
        let vec = Vec::from(&self.remaining()[0..n]);
        self.offset += n;
        debug!("Removed {} bytes from frame buffer, new size: {}",
               n,
               self.remaining().len());
        vec
    }

    fn read_into_string(&mut self, n: usize) -> String {
        let vec = self.read_into_vec(n);
        let s = from_utf8(&vec)
                    .ok()
                    .expect("Attempted to read a string that was not utf8.");
        s.to_owned()
    }

    fn chomp(mut line: String) -> String {
        // This may be suboptimal, but fine for now
        let chomped_length: usize;
        {
            let chars_to_remove: &[char] = &['\r', '\n'];
            let trimmed_line: &str = line.trim_right_matches(chars_to_remove);
            chomped_length = trimmed_line.len();
        }
        line.truncate(chomped_length);
        line
    }
}

enum ReadCommandResult {
    HeartBeat,
    Command(String),
    Incomplete,
}

enum ReadHeaderResult {
    Header(Header),
    EndOfHeaders,
    Incomplete,
}

enum ReadBodyResult {
    Body(Vec<u8>),
    Incomplete,
}

impl Codec {
    pub fn new() -> Codec {
        Codec {
            parse_state: ParseState::new(),
            //   string_pool: Pool::with_size_and_max(DEFAULT_STRING_POOL_SIZE, DEFAULT_STRING_POOL_MAX_SIZE),
            //   header_codec: HeaderCodec::with_pool_size_and_max(DEFAULT_HEADER_CODEC_STRING_POOL_SIZE,
            // DEFAULT_HEADER_CODEC_STRING_POOL_MAX_SIZE)
            header_codec: HeaderCodec::new(),
        }
    }

    // pub fn reset(&mut self) {
    //     self.reset_parse_state();
    // }

    pub fn read_transmission<'a>(&mut self, buffer: &mut FrameBuffer<'a>) -> Option<Transmission> {
        match self.parse_state.section {
            FrameSection::Command => self.resume_parsing_at_command(buffer),
            FrameSection::Headers => self.resume_parsing_at_headers(buffer),
            FrameSection::Body => self.resume_parsing_at_body(buffer),
        }
    }

    fn resume_parsing_at_command<'a>(&mut self,
                                     buffer: &mut FrameBuffer<'a>)
                                     -> Option<Transmission> {
        debug!("Parsing command.");
        match self.read_command(buffer) {
            ReadCommandResult::HeartBeat => Some(Transmission::HeartBeat),
            ReadCommandResult::Command(command_string) => {
                self.parse_state.command = Some(command_string);
                self.parse_state.section = FrameSection::Headers;
                self.resume_parsing_at_headers(buffer)
            }
            ReadCommandResult::Incomplete => None,
        }
    }

    fn resume_parsing_at_headers<'a>(&mut self,
                                     buffer: &mut FrameBuffer<'a>)
                                     -> Option<Transmission> {
        debug!("Parsing headers.");
        loop {
            match self.read_header(buffer) {
                ReadHeaderResult::Header(header) => {
                    self.parse_state.headers.push(header);
                }
                ReadHeaderResult::EndOfHeaders => {
                    self.parse_state.section = FrameSection::Body;
                    return self.resume_parsing_at_body(buffer);
                }
                ReadHeaderResult::Incomplete => return None,
            }
        }
    }

    fn resume_parsing_at_body<'a>(&mut self, buffer: &mut FrameBuffer<'a>) -> Option<Transmission> {
        debug!("Parsing body.");
        match self.read_body(buffer) {
            ReadBodyResult::Body(body_bytes) => {
                let command = match self.parse_state.command.take() {
                    Some(command) => command,
                    None => panic!("No COMMAND found."),
                };
                // Consider making the HeaderList an Option<HeaderList> to allow recycling
                let headers = mem::replace(&mut self.parse_state.headers, header_list![]);
                let body = body_bytes;
                let frame = Frame {
                    command: command,
                    headers: headers,
                    body: body,
                };
                self.reset_parse_state();
                Some(Transmission::CompleteFrame(frame))
            }
            ReadBodyResult::Incomplete => None,
        }
    }

    fn reset_parse_state(&mut self) {
        //self.parse_state.section = FrameSection::Command;
        self.parse_state = ParseState::new(); //TODO: Only for dumb version of codec
    }

    fn read_command<'a>(&mut self, buffer: &mut FrameBuffer<'a>) -> ReadCommandResult {
        match buffer.find_next('\n' as u8) {
            Some(index) => {
                debug!("Found command ending @ index {}", index);
                let num_bytes = index + 1;
                let command = buffer.read_into_string(num_bytes as usize);
                let command = FrameBuffer::chomp(command);
                debug!("Chomped length: {}", command.len());
                if command == "" {
                    debug!("Found HeartBeat");
                    return ReadCommandResult::HeartBeat;
                }
                if command.len() == 1 {
                    debug!("Byte: {}", command.as_bytes()[0]);
                }
                debug!("Command -> '{}'", command);
                ReadCommandResult::Command(command)
            }
            None => ReadCommandResult::Incomplete,
        }
    }

    fn read_header<'a>(&mut self, buffer: &mut FrameBuffer<'a>) -> ReadHeaderResult {
        match buffer.find_next('\n' as u8) {
            Some(index) => {
                debug!("Found header ending @ index {}", index);
                let num_bytes = index + 1;
                let header_string = buffer.read_into_string(num_bytes as usize);
                let header_string = FrameBuffer::chomp(header_string);
                debug!("Header -> '{}'", header_string);
                if header_string == "" {
                    return ReadHeaderResult::EndOfHeaders;
                }
                let header = self.header_codec
                                 .decode(&header_string)
                                 .expect("Invalid header encountered.");
                ReadHeaderResult::Header(header)
            }
            None => ReadHeaderResult::Incomplete,
        }
    }

    fn read_body<'a>(&mut self, buffer: &mut FrameBuffer<'a>) -> ReadBodyResult {
        let maybe_body: Option<Vec<u8>> = match self.parse_state.headers.get_content_length() {
            Some(ContentLength(num_bytes)) => {
                self.read_body_by_content_length(buffer, num_bytes as usize)
            }
            None => self.read_body_by_null_octet(buffer),
        };
        match maybe_body {
            Some(body) => ReadBodyResult::Body(body),
            None => ReadBodyResult::Incomplete,
        }
    }

    fn read_body_by_content_length<'a>(&mut self,
                                       buffer: &mut FrameBuffer<'a>,
                                       content_length: usize)
                                       -> Option<Vec<u8>> {
        debug!("Reading body by content length.");
        debug!("HEADERS: {:?}", self.parse_state.headers);
        let bytes_needed = content_length + 1; // null octet
        if buffer.remaining().len() < bytes_needed {
            debug!("Not enough bytes to form body; needed {}, only had {}.",
                   content_length,
                   buffer.remaining().len());
            return None;
        }
        let mut body = buffer.read_into_vec(bytes_needed);
        body.pop(); // Discard null octet
        debug!("Body -> '{}'", Codec::body_as_string(&body));
        Some(body)
    }

    fn read_body_by_null_octet<'a>(&mut self, buffer: &mut FrameBuffer<'a>) -> Option<Vec<u8>> {
        debug!("Reading body by null octet.");
        match buffer.find_next(0u8) {
            Some(index) => {
                debug!("Found body ending @ index {}", index);
                let num_bytes = index + 1;
                let mut body = buffer.read_into_vec(num_bytes as usize);
                body.pop(); // Discard null octet
                debug!("Body -> '{}'", Codec::body_as_string(&body));
                Some(body)
            }
            None => None,
        }
    }

    fn body_as_string(body: &Vec<u8>) -> &str {
        match from_utf8(&body) {
            Ok(ref s) => *s,
            Err(_) => "<Non-utf8 Binary Content>",
        }
    }
}

impl mai::Codec<Transmission> for Codec {
    fn new() -> Self {
        Codec::new()
    }

    fn encode(&mut self, message: &Transmission, mut buffer: &mut [u8]) -> mai::EncodingResult {
        match message.write(&mut buffer) {
            Ok(BytesWritten(bytes_written)) => Ok(BytesWritten(bytes_written)),
            Err(_) => Err(EncodingError::InsufficientBuffer),
        }
    }

    fn decode(&mut self, buffer: &[u8]) -> mai::DecodingResult<Transmission> {
        // let old_offset = self.parse_state.offset;
        // let bytes_available = buffer.len();
        // debug!("old_offset: {}, available: {}", old_offset, bytes_available);

        // self.parse_state.offset = 0;
        // self.parse_state.section = FrameSection::Command;
        self.parse_state = ParseState::new();

        let buffer = &mut FrameBuffer::from(buffer, self.parse_state.offset);
        let decoding_result = match self.read_transmission(buffer) {
            Some(transmission) => Ok(DecodedFrame::new(transmission, BytesRead(buffer.consumed()))),
            None => {
                debug!("Partial decode occurred. Buffer has {} bytes remaining.", buffer.remaining().len());
                Err(DecodingError::IncompleteFrame)
            },
        };
        // debug!("consumed: {}", buffer.consumed());
        // self.parse_state.offset = bytes_available - buffer.consumed();
        // debug!("new_offset: {}", self.parse_state.offset);
        decoding_result
    }
}
