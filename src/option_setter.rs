use message_builder::MessageBuilder;
use session_builder::SessionBuilder;
use header::Header;
use connection::{HeartBeat, Credentials};

pub trait OptionSetter<T> {
  fn set_option(self, T) -> T;
}

impl <'a> OptionSetter<MessageBuilder<'a>> for Header {
  fn set_option<'b>(self, mut builder: MessageBuilder<'a>) -> MessageBuilder<'a> {
    builder.frame.headers.push(self);
    builder
  }
}

impl <'a> OptionSetter<SessionBuilder<'a>> for Header {
  fn set_option<'b>(self, mut builder: SessionBuilder<'a>) -> SessionBuilder<'a> {
    builder.custom_headers.push(self);
    builder
  }
}

impl <'a> OptionSetter<SessionBuilder<'a>> for HeartBeat {
  fn set_option<'b>(self, mut builder: SessionBuilder<'a>) -> SessionBuilder<'a> {
    builder.heartbeat = self;
    builder
  }
}

impl <'a> OptionSetter<SessionBuilder<'a>> for Credentials<'a> {
  fn set_option<'b>(self, mut builder: SessionBuilder<'a>) -> SessionBuilder<'a> {
    builder.credentials = Some(self);
    builder
  }
}

