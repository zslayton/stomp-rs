use message_builder::MessageBuilder;
use session_builder::SessionBuilder;
use subscription_builder::SubscriptionBuilder;
use header::{Header, SuppressedHeader};
use connection::{HeartBeat, Credentials};
use subscription::AckMode;

pub trait OptionSetter<T> {
  fn set_option(self, T) -> T;
}

impl <'a> OptionSetter<MessageBuilder<'a>> for Header {
  fn set_option(self, mut builder: MessageBuilder<'a>) -> MessageBuilder<'a> {
    builder.frame.headers.push(self);
    builder
  }
}

impl <'a> OptionSetter<MessageBuilder<'a>> for SuppressedHeader<'a> {
  fn set_option(self, mut builder: MessageBuilder<'a>) -> MessageBuilder<'a> {
    let SuppressedHeader(key) = self;
    builder.frame.headers.retain(|header| (*header).get_key() != key);
    builder
  }
}

impl <'a> OptionSetter<SessionBuilder<'a>> for Header {
  fn set_option(self, mut builder: SessionBuilder<'a>) -> SessionBuilder<'a> {
    builder.headers.push(self);
    builder
  }
}

impl <'a> OptionSetter<SessionBuilder<'a>> for HeartBeat {
  fn set_option(self, mut builder: SessionBuilder<'a>) -> SessionBuilder<'a> {
    builder.heartbeat = self;
    builder
  }
}

impl <'a> OptionSetter<SessionBuilder<'a>> for Credentials<'a> {
  fn set_option(self, mut builder: SessionBuilder<'a>) -> SessionBuilder<'a> {
    builder.credentials = Some(self);
    builder
  }
}

impl <'a> OptionSetter<SessionBuilder<'a>> for SuppressedHeader<'a> {
  fn set_option(self, mut builder: SessionBuilder<'a>) -> SessionBuilder<'a> {
    let SuppressedHeader(key) = self;
    builder.headers.retain(|header| (*header).get_key() != key);
    builder
  }
}

impl <'a, 'session, 'sub> OptionSetter<SubscriptionBuilder<'a, 'session, 'sub>> for Header {
  fn set_option(self, mut builder: SubscriptionBuilder<'a, 'session, 'sub>) -> SubscriptionBuilder<'a, 'session, 'sub> {
    builder.headers.push(self);
    builder
  }
}

impl <'b, 'a, 'session, 'sub> OptionSetter<SubscriptionBuilder<'a, 'session, 'sub>> for SuppressedHeader<'b> {
  fn set_option(self, mut builder: SubscriptionBuilder<'a, 'session, 'sub>) -> SubscriptionBuilder<'a, 'session, 'sub> {
    let SuppressedHeader(key) = self;
    builder.headers.retain(|header| (*header).get_key() != key);
    builder
  }
}

impl <'a, 'session, 'sub> OptionSetter<SubscriptionBuilder<'a, 'session, 'sub>> for AckMode {
  fn set_option(self, mut builder: SubscriptionBuilder<'a, 'session, 'sub>) -> SubscriptionBuilder<'a, 'session, 'sub> {
    builder.ack_mode = self;
    builder
  }
}

