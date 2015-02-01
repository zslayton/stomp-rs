use session::Session;
use frame::Frame;
use option_setter::OptionSetter;
use std::old_io::IoResult;

pub struct MessageBuilder <'a> {
  pub session: &'a Session<'a>,
  pub frame: Frame
}

impl <'a> MessageBuilder <'a> {
  #[allow(dead_code)] 
  pub fn send(self) -> IoResult<()> {
    self.session.send(self.frame)
  }

  #[allow(dead_code)] 
  pub fn with<T>(self, option_setter: T) -> MessageBuilder<'a> where T: OptionSetter<MessageBuilder<'a>> {
    option_setter.set_option(self) 
  } 
}


