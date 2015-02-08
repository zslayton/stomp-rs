use frame::Frame;
use frame::ToFrameBody;
use message_builder::MessageBuilder;
use std::old_io::IoResult;
use header::Header;
use session::Session;

pub struct Transaction<'a, 'session: 'a> {
  pub id: String, 
  pub session: &'a mut Session<'session>
}

impl <'a, 'session> Transaction<'a, 'session> {
  pub fn new<'b>(id: u32, session: &'b mut Session<'session>) -> Transaction<'b, 'session> {
    Transaction {
      id: format!("tx/{}",id),
      session: session,
    }
  }

  pub fn message<'b, T: ToFrameBody> (&'b mut self, destination: & str, body_convertible: T) -> MessageBuilder<'b, 'session> {
    let mut send_frame = Frame::send(destination, body_convertible.to_frame_body());
    send_frame.headers.push(Header::new("transaction", self.id.as_slice()));
    MessageBuilder {
     session: self.session,
     frame: send_frame
    }
  }

  pub fn begin<'b>(&'b self) -> IoResult<()> {
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
