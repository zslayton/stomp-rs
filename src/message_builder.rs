use session::Session;
use frame::Frame;
use option_setter::OptionSetter;
use std::io::Result;

pub struct MessageBuilder <'a, 'session: 'a> {
  pub session: &'a mut Session<'session>,
  pub frame: Frame
}

impl <'a, 'session> MessageBuilder <'a, 'session> {
  #[allow(dead_code)] 
  pub fn send(self) -> Result<()> {
    self.session.send(self.frame)
  }

  #[allow(dead_code)] 
  pub fn with<T>(self, option_setter: T) -> MessageBuilder<'a, 'session> where T: OptionSetter<MessageBuilder<'a, 'session>> {
    option_setter.set_option(self) 
  } 
}


