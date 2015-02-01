use message_builder::MessageBuilder;
use header::Header;

pub trait OptionSetter<T> {
  fn set_option(self, T) -> T;
}

impl <'a> OptionSetter<MessageBuilder<'a>> for Header {
  fn set_option<'b>(self, mut builder: MessageBuilder<'a>) -> MessageBuilder<'a> {
    builder.frame.headers.push(self);
    builder
  }
}
