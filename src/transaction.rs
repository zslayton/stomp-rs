use frame::Frame;
use std::old_io::IoResult;
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
    self.session.send(send_frame)
  }

  pub fn send_text(&mut self, topic: &str, body: &str) -> IoResult<()> {
    self.send_bytes(topic, "text/plain", body.as_bytes())
  }

  pub fn begin(&mut self) -> IoResult<()> {
    let begin_frame = Frame::begin(self.id.as_slice());
    self.session.send(begin_frame)
  }

  pub fn commit(self) -> IoResult<()> {
    let commit_frame = Frame::commit(self.id.as_slice());
    self.session.send(commit_frame)
  }

  pub fn abort(self) -> IoResult<()> {
    let abort_frame = Frame::abort(self.id.as_slice());
    self.session.send(abort_frame)
  }
}
