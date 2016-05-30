use session::Session;
use frame::Frame;
use option_setter::OptionSetter;
use std::io::Result;
use handler::Handler;

pub struct MessageBuilder<'builder, 'session: 'builder, H: 'session> where H: Handler {
    pub session: &'builder mut Session<'session, H>,
    pub frame: Frame,
}

impl<'builder, 'session, 'context, H> MessageBuilder<'builder, 'session, H> where H: Handler {
    #[allow(dead_code)]
    pub fn send(self) -> Result<()> {
        self.session.send(self.frame)
    }

    #[allow(dead_code)]
    pub fn with<T>(self, option_setter: T) -> MessageBuilder<'builder, 'session, H>
        where T: OptionSetter<MessageBuilder<'builder, 'session, H>>
    {
        option_setter.set_option(self)
    }
}
