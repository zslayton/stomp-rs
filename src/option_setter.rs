use message_builder::MessageBuilder;
use session_builder::SessionBuilder;
use subscription_builder::SubscriptionBuilder;
use header::{Header, SuppressedHeader};
use connection::{HeartBeat, Credentials};
use subscription::AckMode;
use session::{ToFrameHandler, ReceiptHandler};

pub trait OptionSetter<T> {
  fn set_option(self, T) -> T;
}

impl <'a, 'b> OptionSetter<MessageBuilder<'a, 'b>> for Header {
  fn set_option(self, mut builder: MessageBuilder<'a, 'b>) -> MessageBuilder<'a, 'b> {
    builder.frame.headers.push(self);
    builder
  }
}

impl <'a, 'b> OptionSetter<MessageBuilder<'a, 'b>> for SuppressedHeader<'a> {
  fn set_option(self, mut builder: MessageBuilder<'a, 'b>) -> MessageBuilder<'a, 'b> {
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

impl <'a, 'session, T> OptionSetter<MessageBuilder<'a, 'session>> for ReceiptHandler<'session, T> where T : ToFrameHandler<'session> {
  fn set_option(self, mut builder: MessageBuilder<'a, 'session>) -> MessageBuilder<'a, 'session> {
    let next_id = builder.session.generate_receipt_id();
    let receipt_id = format!("message/{}", next_id);
    let ReceiptHandler(handler_convertible) = self;
    let handler = handler_convertible.to_frame_handler();
    builder.frame.headers.push(Header::new("receipt", receipt_id.as_slice())); 
    builder.session.receipt_handlers.insert(receipt_id.to_string(), handler);;
    builder
  }
}

