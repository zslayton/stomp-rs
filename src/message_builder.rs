use session::{Session, ReceiptHandler, ReceiptHandlerFn, ReceiptRequest, OutstandingReceiptHandler};
use frame::Frame;
use option_setter::OptionSetter;
use std::io::Result;
use handler::Handler;

pub struct MessageBuilder<'builder, 'session: 'builder, H: 'session> where H: Handler {
    pub session: &'builder mut Session<'session, H>,
    pub frame: Frame,
    pub receipt_request: Option<ReceiptRequest<H>>
}

impl<'builder, 'session, 'context, H> MessageBuilder<'builder, 'session, H> where H: Handler {
    pub fn new(session: &'builder mut Session<'session, H>, frame: Frame) -> Self {
        MessageBuilder {
            session: session,
            frame: frame,
            receipt_request: None
        }
    }

    #[allow(dead_code)]
    pub fn send(self) -> Result<()> {
        if self.receipt_request.is_some() {
            let request = self.receipt_request.unwrap();
            self.session
                .state()
                .outstanding_receipts
                .insert(
                    request.id,
                    OutstandingReceiptHandler::new(
                        self.frame.clone(),
                        request.handler
                    )
                );
        }
        self.session.send(self.frame)
    }

    #[allow(dead_code)]
    pub fn with<T>(self, option_setter: T) -> MessageBuilder<'builder, 'session, H>
        where T: OptionSetter<MessageBuilder<'builder, 'session, H>>
    {
        option_setter.set_option(self)
    }
}
