use frame::Frame;
use std::io::IoResult;
use header::Header;
use session::Session;

pub struct Transaction<'a> {
  pub id: String, 
  pub session: &'a mut Session
}

impl <'a> Transaction<'a> {
  pub fn new<'a>(id: uint, session: &'a mut Session) -> Transaction<'a> {
    Transaction {
      id: format!("tx/{}",id),
      session: session,
    }
  }

  pub fn send_bytes(&mut self, topic: &str, mime_type: &str, body: &[u8]) -> IoResult<()> {
    let mut send_frame = Frame::send(topic, mime_type, body);
    send_frame.headers.push(
      Header::encode_key_value("transaction", self.id.as_slice())
    );
    Ok(try!(send_frame.write(&mut self.session.connection.writer)))
  }

  pub fn send_text(&mut self, topic: &str, body: &str) -> IoResult<()> {
    Ok(try!(self.send_bytes(topic, "text/plain", body.as_bytes())))
  }

  pub fn begin(&mut self) -> IoResult<()> {
    let begin_frame = Frame::begin(self.id.as_slice());
    Ok(try!(begin_frame.write(&mut self.session.connection.writer)))
  }

  pub fn commit(&mut self) -> IoResult<()> {
    let commit_frame = Frame::commit(self.id.as_slice());
    Ok(try!(commit_frame.write(&mut self.session.connection.writer)))
  }

  pub fn abort(&mut self) -> IoResult<()> {
    let abort_frame = Frame::commit(self.id.as_slice());
    Ok(try!(abort_frame.write(&mut self.session.connection.writer)))
  }
}
