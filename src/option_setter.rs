use message_builder::MessageBuilder;
use session_builder::SessionBuilder;
use subscription_builder::SubscriptionBuilder;
use header::{Header, SuppressedHeader, ContentType};
use connection::{HeartBeat, Credentials, OwnedCredentials};
use subscription::AckMode;
use session::{ToFrameHandler, ReceiptHandler};
use handler::Handler;

pub trait OptionSetter<T> {
    fn set_option(self, T) -> T;
}

impl <'builder, 'session, H> OptionSetter<MessageBuilder<'builder, 'session, H>> for Header where H: Handler {
  fn set_option(self, mut builder: MessageBuilder<'builder, 'session, H>) -> MessageBuilder<'builder, 'session, H> {
    builder.frame.headers.push(self);
    builder
  }
}

impl <'a, 'builder, 'session, H> OptionSetter<MessageBuilder<'builder, 'session, H>> for SuppressedHeader<'a> where H: Handler {
  fn set_option(self, mut builder: MessageBuilder<'builder, 'session, H>) -> MessageBuilder<'builder, 'session, H> {
    let SuppressedHeader(key) = self;
    builder.frame.headers.retain(|header| (*header).get_key() != key);
    builder
  }
}

impl <'a, 'builder, 'session, H> OptionSetter<MessageBuilder<'builder, 'session, H>> for ContentType<'a> where H: Handler {
  fn set_option(self, mut builder: MessageBuilder<'builder, 'session, H>) -> MessageBuilder<'builder, 'session, H> {
    let ContentType(content_type) = self;
    builder.frame.headers.push(Header::new("content-type", content_type));
    builder
  }
}

impl<'a, H: 'a> OptionSetter<SessionBuilder<'a, H>> for Header where H: Handler {
    fn set_option(self, mut builder: SessionBuilder<H>) -> SessionBuilder<H> {
        builder.config.headers.push(self);
        builder
    }
}

impl<'a, H: 'a> OptionSetter<SessionBuilder<'a, H>> for HeartBeat where H: Handler {
    fn set_option(self, mut builder: SessionBuilder<H>) -> SessionBuilder<H> {
        builder.config.heartbeat = self;
        builder
    }
}

impl<'a, 'b, H: 'a> OptionSetter<SessionBuilder<'a, H>> for Credentials<'b> where H: Handler {
    fn set_option(self, mut builder: SessionBuilder<H>) -> SessionBuilder<H> {
        builder.config.credentials = Some(OwnedCredentials::from(self));
        builder
    }
}

impl<'a, 'b, H: 'a> OptionSetter<SessionBuilder<'a, H>> for SuppressedHeader<'b> where H: Handler {
    fn set_option(self, mut builder: SessionBuilder<H>) -> SessionBuilder<H> {
        let SuppressedHeader(key) = self;
        builder.config.headers.retain(|header| (*header).get_key() != key);
        builder
    }
}

impl <'builder, 'session, 'context, H> OptionSetter<SubscriptionBuilder<'builder, 'session, H>> for Header where H: Handler {
  fn set_option(self, mut builder: SubscriptionBuilder<'builder, 'session, H>) -> SubscriptionBuilder<'builder, 'session, H> {
    builder.headers.push(self);
    builder
  }
}

impl <'builder, 'session, 'context, 'a, H> OptionSetter<SubscriptionBuilder<'builder, 'session, H>> for SuppressedHeader<'a> where H: Handler {
  fn set_option(self, mut builder: SubscriptionBuilder<'builder, 'session, H>) -> SubscriptionBuilder<'builder, 'session, H> {
    let SuppressedHeader(key) = self;
    builder.headers.retain(|header| (*header).get_key() != key);
    builder
  }
}

impl <'builder, 'session, 'context, H> OptionSetter<SubscriptionBuilder<'builder, 'session, H>> for AckMode  where H: Handler {
  fn set_option(self, mut builder: SubscriptionBuilder<'builder, 'session, H>) -> SubscriptionBuilder<'builder, 'session, H> {
    builder.ack_mode = self;
    builder
  }
}

impl <'builder, 'session, 'context, T, H> OptionSetter<MessageBuilder<'builder, 'session, H>> for ReceiptHandler<T> where T : ToFrameHandler, H: Handler {
  fn set_option(self, mut builder: MessageBuilder<'builder, 'session, H>) -> MessageBuilder<'builder, 'session, H> {
    let next_id = builder.session.generate_receipt_id();
    let receipt_id = format!("message/{}", next_id);
    let handler_convertible = self.handler;
    let handler = handler_convertible.to_frame_handler();
    builder.frame.headers.push(Header::new("receipt", receipt_id.as_ref()));
// builder.session.context.session().receipt_handlers.insert(receipt_id.to_string(), handler);
    builder
  }
}

impl <'builder, 'session, 'context, T, H> OptionSetter<SubscriptionBuilder<'builder, 'session, H>> for ReceiptHandler<T> where T : ToFrameHandler, H: Handler {
  fn set_option(self, mut builder: SubscriptionBuilder<'builder, 'session, H>) -> SubscriptionBuilder<'builder, 'session, H> {
    let next_id = builder.session.generate_receipt_id();
    let receipt_id = format!("message/{}", next_id);
    let handler_convertible = self.handler;
    let handler = handler_convertible.to_frame_handler();
    builder.headers.push(Header::new("receipt", receipt_id.as_ref()));
// builder.session.context.session().receipt_handlers.insert(receipt_id.to_string(), handler);
    builder
  }
}
