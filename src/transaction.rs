use frame::Frame;
use frame::ToFrameBody;
use message_builder::MessageBuilder;
use std::io::Result;
use header::Header;
use handler::Handler;
use session::{Session, SessionContext};

pub struct Transaction<'tx, 'session: 'tx, H: 'session> where H: Handler {
    pub id: String,
    pub session: &'tx mut Session<'session, H>,
}

impl<'tx, 'session, 'stream, H> Transaction<'tx, 'session, H> where H: Handler {
    pub fn new(session: &'tx mut Session<'session, H>)
               -> Transaction<'tx, 'session, H> {
        Transaction {
            id: format!("tx/{}", session.generate_transaction_id()),
            session: session,
        }
    }

    pub fn message<'builder, T: ToFrameBody>(&'builder mut self,
                                             destination: &str,
                                             body_convertible: T)
                                             -> MessageBuilder<'builder, 'session, H> {
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
