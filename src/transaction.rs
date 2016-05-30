use frame::Frame;
use frame::ToFrameBody;
use message_builder::MessageBuilder;
use std::io::Result;
use header::Header;
use session::{Session, SessionContext};

pub struct Transaction<'tx, 'session: 'tx> {
    pub id: String,
    pub session: &'tx mut Session<'session>,
}

impl<'tx, 'session, 'stream> Transaction<'tx, 'session> {
    pub fn new(session: &'tx mut Session<'session>)
               -> Transaction<'tx, 'session> {
        Transaction {
            id: format!("tx/{}", session.generate_transaction_id()),
            session: session,
        }
    }

    pub fn message<'builder, T: ToFrameBody>(&'builder mut self,
                                             destination: &str,
                                             body_convertible: T)
                                             -> MessageBuilder<'builder, 'session> {
        let mut send_frame = Frame::send(destination, body_convertible.to_frame_body());
        send_frame.headers.push(Header::new("transaction", self.id.as_ref()));
        MessageBuilder {
            session: self.session,
            frame: send_frame,
        }
    }

    // TODO: See if it's feasible to do this via command_sender

    pub fn begin(&mut self) -> Result<()> {
        let begin_frame = Frame::begin(self.id.as_ref());
        self.session.send(begin_frame)
    }

    pub fn commit(self) -> Result<()> {
        let commit_frame = Frame::commit(self.id.as_ref());
        self.session.send(commit_frame)
    }

    pub fn abort(self) -> Result<()> {
        let abort_frame = Frame::abort(self.id.as_ref());
        self.session.send(abort_frame)
    }
}
