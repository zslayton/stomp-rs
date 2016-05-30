use message_builder::MessageBuilder;
use session_builder::SessionBuilder;
use subscription_builder::SubscriptionBuilder;
use header::{Header, SuppressedHeader, ContentType};
use connection::{HeartBeat, Credentials, OwnedCredentials};
use subscription::AckMode;
use session::{ToFrameHandler, ReceiptHandler};

pub trait OptionSetter<T> {
    fn set_option(self, T) -> T;
}

impl <'builder, 'session> OptionSetter<MessageBuilder<'builder, 'session>> for Header {
  fn set_option(self, mut builder: MessageBuilder<'builder, 'session>) -> MessageBuilder<'builder, 'session> {
    builder.frame.headers.push(self);
    builder
  }
}

impl <'a, 'builder, 'session> OptionSetter<MessageBuilder<'builder, 'session>> for SuppressedHeader<'a> {
  fn set_option(self, mut builder: MessageBuilder<'builder, 'session>) -> MessageBuilder<'builder, 'session> {
    let SuppressedHeader(key) = self;
    builder.frame.headers.retain(|header| (*header).get_key() != key);
    builder
  }
}

impl <'a, 'builder, 'session> OptionSetter<MessageBuilder<'builder, 'session>> for ContentType<'a> {
  fn set_option(self, mut builder: MessageBuilder<'builder, 'session>) -> MessageBuilder<'builder, 'session> {
    let ContentType(content_type) = self;
    builder.frame.headers.push(Header::new("content-type", content_type));
    builder
  }
}

impl<'a> OptionSetter<SessionBuilder<'a>> for Header {
    fn set_option(self, mut builder: SessionBuilder) -> SessionBuilder {
        builder.config.headers.push(self);
        builder
    }
}

impl<'a> OptionSetter<SessionBuilder<'a>> for HeartBeat {
    fn set_option(self, mut builder: SessionBuilder) -> SessionBuilder {
        builder.config.heartbeat = self;
        builder
    }
}

impl<'a, 'b> OptionSetter<SessionBuilder<'a>> for Credentials<'b> {
    fn set_option(self, mut builder: SessionBuilder) -> SessionBuilder {
        builder.config.credentials = Some(OwnedCredentials::from(self));
        builder
    }
}

impl<'a, 'b> OptionSetter<SessionBuilder<'a>> for SuppressedHeader<'b> {
    fn set_option(self, mut builder: SessionBuilder) -> SessionBuilder {
        let SuppressedHeader(key) = self;
        builder.config.headers.retain(|header| (*header).get_key() != key);
        builder
    }
}

impl <'builder, 'session, 'context> OptionSetter<SubscriptionBuilder<'builder, 'session>> for Header {
  fn set_option(self, mut builder: SubscriptionBuilder<'builder, 'session>) -> SubscriptionBuilder<'builder, 'session> {
    builder.headers.push(self);
    builder
  }
}

impl <'builder, 'session, 'context, 'a> OptionSetter<SubscriptionBuilder<'builder, 'session>> for SuppressedHeader<'a> {
  fn set_option(self, mut builder: SubscriptionBuilder<'builder, 'session>) -> SubscriptionBuilder<'builder, 'session> {
    let SuppressedHeader(key) = self;
    builder.headers.retain(|header| (*header).get_key() != key);
    builder
  }
}

impl <'builder, 'session, 'context> OptionSetter<SubscriptionBuilder<'builder, 'session>> for AckMode {
  fn set_option(self, mut builder: SubscriptionBuilder<'builder, 'session>) -> SubscriptionBuilder<'builder, 'session> {
    builder.ack_mode = self;
    builder
  }
}

impl <'builder, 'session, 'context, T> OptionSetter<MessageBuilder<'builder, 'session>> for ReceiptHandler<T> where T : ToFrameHandler {
  fn set_option(self, mut builder: MessageBuilder<'builder, 'session>) -> MessageBuilder<'builder, 'session> {
    let next_id = builder.session.generate_receipt_id();
    let receipt_id = format!("message/{}", next_id);
    let handler_convertible = self.handler;
    let handler = handler_convertible.to_frame_handler();
    builder.frame.headers.push(Header::new("receipt", receipt_id.as_ref()));
// builder.session.context.session().receipt_handlers.insert(receipt_id.to_string(), handler);
    builder
  }
}

impl <'builder, 'session, 'context, T> OptionSetter<SubscriptionBuilder<'builder, 'session>> for ReceiptHandler<T> where T : ToFrameHandler {
  fn set_option(self, mut builder: SubscriptionBuilder<'builder, 'session>) -> SubscriptionBuilder<'builder, 'session> {
    let next_id = builder.session.generate_receipt_id();
    let receipt_id = format!("message/{}", next_id);
    let handler_convertible = self.handler;
    let handler = handler_convertible.to_frame_handler();
    builder.headers.push(Header::new("receipt", receipt_id.as_ref()));
// builder.session.context.session().receipt_handlers.insert(receipt_id.to_string(), handler);
    builder
  }
}
