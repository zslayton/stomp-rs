use header::{Header, HeaderList};
use frame::{Frame, Transmission};
use bytes::BytesMut;
use frame::Command;
use tokio_io::codec::{Encoder, Decoder};
use nom::{line_ending, anychar};

named!(parse_server_command(&[u8]) -> Command,
       alt!(
           map!(tag!("CONNECTED"), |_| Command::Connected) |
           map!(tag!("MESSAGE"), |_| Command::Message) |
           map!(tag!("RECEIPT"), |_| Command::Receipt) |
           map!(tag!("ERROR"), |_| Command::Error)
       )
);
named!(parse_header_character(&[u8]) -> char,
       alt_complete!(
           map!(tag!("\\n"), |_| '\n') |
           map!(tag!("\\r"), |_| '\r') |
           map!(tag!("\\c"), |_| ':') |
           map!(tag!("\\\\"), |_| '\\') |
           anychar
       )
);
named!(parse_header(&[u8]) -> Header,
       map!(
           do_parse!(
               k: flat_map!(is_not!(":\r\n"), many1!(parse_header_character)) >>
               tag!(":") >>
               v: flat_map!(is_not!("\r\n"), many1!(parse_header_character))>>
               line_ending >>
               (k, v)
           ),
           |(k, v)| {
               Header::new_raw(k.into_iter().collect::<String>(), v.into_iter().collect::<String>())
           }
       )
);
named!(parse_frame(&[u8]) -> Frame,
       map!(
           do_parse!(
               cmd: parse_server_command >>
               line_ending >>
               headers: many0!(parse_header) >>
               line_ending >>
               body: many0!(is_not!("\0")) >>
               tag!("\0") >>
               (cmd, headers, body)
           ),
           |(cmd, headers, body)| {
               let body = if body.len() == 0 {
                   &[]
               } else {
                   body.into_iter().nth(0).unwrap()
               };
               Frame {
                   command: cmd,
                   headers: HeaderList { headers },
                   body: body.into()
               }
           }
       )
);
named!(parse_transmission(&[u8]) -> Transmission,
       alt!(
           map!(many1!(line_ending), |_| Transmission::HeartBeat) |
           map!(parse_frame, |f| Transmission::CompleteFrame(f))
       )
);
pub struct Codec;

impl Encoder for Codec {
    type Item = Transmission;
    type Error = ::std::io::Error;
    fn encode(&mut self, item: Transmission, buffer: &mut BytesMut) -> Result<(), ::std::io::Error> {
        item.write(buffer);
        Ok(())
    }
}
impl Decoder for Codec {
    type Item = Transmission;
    type Error = ::std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Transmission>, ::std::io::Error> {
        use nom::IResult;
        use std::io::{Error, ErrorKind};

        trace!("decoding data: {:?}", src);
        let (point, data) = match parse_transmission(src) {
            IResult::Done(rest, data) => {
                (rest.len(), data)
            },
            IResult::Error(e) => {
                warn!("parse error: {:?}", e);
                return Err(Error::new(ErrorKind::Other, format!("parse error: {}", e)));
            },
            IResult::Incomplete(_) => return Ok(None)
        };
        let len = src.len().saturating_sub(point);
        src.split_to(len);
        Ok(Some(data))
    }
}
