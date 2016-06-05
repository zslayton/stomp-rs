use session::Session;
use frame::Frame;

pub trait Handler: Send + Sized {
    fn on_connected(&mut self, _session: &mut Session<Self>, _frame: &Frame) {
        debug!("Connected.");
    }

    fn on_error(&mut self, _session: &mut Session<Self>, frame: &Frame) {
        error!("Error:\n{:?}", frame);
    }

    fn on_disconnected(&mut self, _session: &mut Session<Self>) {
        debug!("Disconnected.");
    }
}
