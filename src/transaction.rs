use frame::Frame;
use std::io::IoResult;
use header::Header;
use session::Session;

pub struct Transaction<'a, 'b: 'a> {
  pub id: String, 
  pub session: &'a mut Session<'b>
}

impl <'a, 'b> Transaction<'a, 'b> {
  pub fn new(id: u32, session: &'a mut Session<'b>) -> Transaction<'a, 'b> {
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
    //Ok(try!(self.session.send(send_frame)))
    Ok(self.session.send(send_frame))
  }

  pub fn send_text(&mut self, topic: &str, body: &str) -> IoResult<()> {
    Ok(try!(self.send_bytes(topic, "text/plain", body.as_bytes())))
  }

  pub fn begin(&mut self) -> IoResult<()> {
    let begin_frame = Frame::begin(self.id.as_slice());
    //Ok(try!(self.session.send(begin_frame)))
    Ok(self.session.send(begin_frame))
  }

  pub fn commit(&mut self) -> IoResult<()> {
    let commit_frame = Frame::commit(self.id.as_slice());
    //Ok(try!(self.session.send(commit_frame)))
    Ok(self.session.send(commit_frame))
  }

  pub fn abort(&mut self) -> IoResult<()> {
    let abort_frame = Frame::abort(self.id.as_slice());
    //Ok(try!(self.session.send(abort_frame)))
    Ok(self.session.send(abort_frame))
  }
}
