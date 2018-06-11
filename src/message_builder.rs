use session::{Session, ReceiptRequest, OutstandingReceipt};
use frame::Frame;
use option_setter::OptionSetter;

pub struct MessageBuilder<'a> {
    pub session: &'a mut Session,
    pub frame: Frame,
    pub receipt_request: Option<ReceiptRequest>
}

impl<'a> MessageBuilder<'a> {
    pub fn new(session: &'a mut Session, frame: Frame) -> Self {
        MessageBuilder {
            session: session,
            frame: frame,
            receipt_request: None
        }
    }

    #[allow(dead_code)]
    pub fn send(self) {
        if self.receipt_request.is_some() {
            let request = self.receipt_request.unwrap();
            self.session.state.outstanding_receipts.insert(
                request.id,
                OutstandingReceipt::new(
                    self.frame.clone()
                )
            );
        }
        self.session.send_frame(self.frame)
    }

    #[allow(dead_code)]
    pub fn with<T>(self, option_setter: T) -> MessageBuilder<'a>
        where T: OptionSetter<MessageBuilder<'a>>
    {
        option_setter.set_option(self)
    }
}
