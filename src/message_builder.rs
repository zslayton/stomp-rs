use session::Session;
use frame::Frame;
use option_setter::OptionSetter;
use std::io::Result;

pub struct MessageBuilder<'builder, 'session: 'builder, 'context: 'session> {
    pub session: &'builder mut Session<'session, 'context>,
    pub frame: Frame,
}

impl<'builder, 'session, 'context> MessageBuilder<'builder, 'session, 'context> {
    #[allow(dead_code)]
    pub fn send(self) -> Result<()> {
        self.session.send(self.frame)
    }

    #[allow(dead_code)]
    pub fn with<T>(self, option_setter: T) -> MessageBuilder<'builder, 'session, 'context>
        where T: OptionSetter<MessageBuilder<'builder, 'session, 'context>>
    {
        option_setter.set_option(self)
    }
}
