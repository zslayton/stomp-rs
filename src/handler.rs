use session::Session;
use frame::Frame;
use subscription::AckOrNack::{self, Ack};

pub trait Handler: Send {
    fn on_connected(&mut self, _session: &mut Session, _frame: &Frame) {
        debug!("Connected.");
    }

    fn on_receipt(&mut self, _session: &mut Session, receipt: &Frame) {
        debug!("Received a Receipt:\n{:?}", receipt);
    }

    fn on_message(&mut self, _session: &mut Session, frame: &Frame) -> AckOrNack {
        debug!("Received a Message:\n{:?}", frame);
        Ack
    }

    fn on_error(&mut self, _session: &mut Session, frame: &Frame) {
        error!("Error:\n{:?}", frame);
    }

    fn on_disconnected(&mut self, _session: &mut Session) {
        debug!("Disconnected.");
    }

    fn on_before_send(&mut self, _frame: &mut Frame) {
        debug!("On Before Send");
    }

    fn on_before_receive(&mut self, _frame: &mut Frame) {
        debug!("On Before Receive.");
    }
}
