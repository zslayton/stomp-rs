use session::Session;
use subscription::{Subscription, MessageHandler, AckMode};
use frame::Frame;
use header::HeaderList;
use option_setter::OptionSetter;
use std::old_io::IoResult;

pub struct SubscriptionBuilder <'a, 'session: 'a, 'sub: 'session> {
  pub session: &'a mut Session<'session>,
  pub destination: &'a str,
  pub ack_mode: AckMode,
  pub handler: Box<MessageHandler + 'sub>,
  pub headers: HeaderList
}

impl <'a, 'session, 'sub> SubscriptionBuilder <'a, 'session, 'sub> {

  #[allow(dead_code)] 
  pub fn start(mut self) -> IoResult<String> {
    let next_id = self.session.generate_subscription_id();
    let subscription = Subscription::new(next_id, self.destination, self.ack_mode, self.handler);
    let mut subscribe_frame = Frame::subscribe(subscription.id.as_slice(), self.destination, self.ack_mode);

    subscribe_frame.headers.concat(&mut self.headers);
   
    try!(self.session.send(subscribe_frame));
    debug!("Registering callback for subscription id '{}' from builder", subscription.id);
    let id_to_return = subscription.id.to_string();
    self.session.subscriptions.insert(subscription.id.to_string(), subscription);
    Ok(id_to_return)
  }

  #[allow(dead_code)] 
  pub fn with<T>(self, option_setter: T) -> SubscriptionBuilder<'a, 'session, 'sub> where T: OptionSetter<SubscriptionBuilder<'a, 'session, 'sub>> {
    option_setter.set_option(self) 
  } 
}


