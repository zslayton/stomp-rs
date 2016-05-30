use session::Session;
use subscription::{Subscription, AckMode, MessageHandler};
use handler::Handler;
use frame::Frame;
use header::HeaderList;
use option_setter::OptionSetter;
use std::io::Result;

pub struct SubscriptionBuilder<'builder, 'session: 'builder, H: 'session> where H: Handler {
    pub session: &'builder mut Session<'session, H>,
    pub destination: String,
    pub ack_mode: AckMode,
    pub handler: MessageHandler<H>,
    pub headers: HeaderList,
}

impl<'builder, 'session, 'context, H> SubscriptionBuilder<'builder, 'session, H> where H: Handler {
    #[allow(dead_code)]
    pub fn start(mut self) -> Result<String> {
        let next_id = self.session.generate_subscription_id();
        let subscription = Subscription::new(next_id,
                                             &self.destination,
                                             self.ack_mode,
                                             self.headers.clone(),
                                             self.handler);
        let mut subscribe_frame = Frame::subscribe(&subscription.id,
                                                   &self.destination,
                                                   self.ack_mode);

        subscribe_frame.headers.concat(&mut self.headers);

        try!(self.session.send(subscribe_frame));
        debug!("Registering callback for subscription id '{}' from builder",
               subscription.id);
        let id_to_return = subscription.id.to_string();
        self.session
            .state()
            .subscriptions
            .insert(subscription.id.to_string(), subscription);
        Ok(id_to_return)
    }

    #[allow(dead_code)]
    pub fn with<T>(self, option_setter: T) -> SubscriptionBuilder<'builder, 'session, H>
        where T: OptionSetter<SubscriptionBuilder<'builder, 'session, H>>
    {
        option_setter.set_option(self)
    }
}
