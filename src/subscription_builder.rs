use session::Session;
use subscription::Subscription;
use frame::Frame;
use option_setter::OptionSetter;
use std::old_io::IoResult;

pub struct SubscriptionBuilder <'a, 'session: 'a, 'sub: 'session> {
  pub session: &'a mut Session<'session>,
  pub frame: Frame,
  pub subscription: Subscription<'sub> 
}

impl <'a, 'session, 'sub> SubscriptionBuilder <'a, 'session, 'sub> {

  pub fn new(session: &'a mut Session<'session>, frame: Frame, sub: Subscription<'sub>) -> SubscriptionBuilder <'a, 'session, 'sub> {
   SubscriptionBuilder {
      session: session,
      frame: frame,
      subscription: sub
    }
  } 

  #[allow(dead_code)] 
  pub fn create(self) -> IoResult<String> {
    let id_to_return = self.subscription.id.to_string();
    debug!("Registering callback for subscription id '{}' from builder", self.subscription.id);
    self.session.subscriptions.insert(self.subscription.id.to_string(), self.subscription);
    try!(self.session.send(self.frame));
    Ok(id_to_return)
  }

  #[allow(dead_code)] 
  pub fn with<T>(self, option_setter: T) -> SubscriptionBuilder<'a, 'session, 'sub> where T: OptionSetter<SubscriptionBuilder<'a, 'session, 'sub>> {
    option_setter.set_option(self) 
  } 
}


