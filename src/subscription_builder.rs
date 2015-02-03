use session::Session;
use subscription::Subscription;
use frame::Frame;
use option_setter::OptionSetter;
use std::old_io::IoResult;

pub struct SubscriptionBuilder <'a> {
  pub session: &'a mut Session<'a>,
  pub frame: Frame,
  pub subscription: Subscription<'a> 
}

impl <'a> SubscriptionBuilder <'a> {
  pub fn new(session: &'a mut Session<'a>, frame: Frame, sub: Subscription<'a>) -> SubscriptionBuilder <'a> {
   SubscriptionBuilder {
      session: session,
      frame: frame,
      subscription: sub
    }
  } 

  #[allow(dead_code)] 
  pub fn create(mut self) -> IoResult<String> {
    try!(self.session.send(self.frame));
    debug!("Registering callback for subscription id '{}' from builder", self.subscription.id);
    let id_to_return = self.subscription.id.to_string();
    self.session.subscriptions.insert(self.subscription.id.to_string(), self.subscription);
    Ok(id_to_return)
  }

  #[allow(dead_code)] 
  pub fn with<T>(self, option_setter: T) -> SubscriptionBuilder<'a> where T: OptionSetter<SubscriptionBuilder<'a>> {
    option_setter.set_option(self) 
  } 
}


