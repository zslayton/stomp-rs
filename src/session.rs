use std::collections::hash_map::HashMap;
use std::io::Result;
use std::io::Error;
use std::io::ErrorKind::Other;
use std::marker::PhantomData;
use connection::Connection;
use connection;
use subscription::AckMode;
use subscription::AckOrNack;
use subscription::AckOrNack::{Ack, Nack};
use subscription::{Subscription, MessageHandler, ToMessageHandler};
use frame::Frame;
use frame::ToFrameBody;
use frame::Transmission::{HeartBeat, CompleteFrame, ConnectionClosed};
use header;
use header::Header;
use header::HeaderList;
use header::ReceiptId;
use header::StompHeaderSet;
use transaction::Transaction;
use session_builder::{SessionBuilder, SessionConfig};
use message_builder::MessageBuilder;
use subscription_builder::SubscriptionBuilder;
use timeout::Timeout;

use handler::Handler;

use protocol::StompProtocol;
use mio;

use mai;
use mai::Context;
use mai::context::StreamHandle;
use mai::ProtocolEngine;

use session_manager::SessionManager;

pub trait FrameHandler: Send {
    fn on_frame(&mut self, &Frame);
}

pub trait FrameHandlerMut: Send {
    fn on_frame(&mut self, &mut Frame);
}

impl<F> FrameHandler for F
    where F: FnMut(&Frame) + Send
{
    fn on_frame(&mut self, frame: &Frame) {
        self(frame)
    }
}

impl<F> FrameHandlerMut for F
    where F: FnMut(&mut Frame) + Send
{
    fn on_frame(&mut self, frame: &mut Frame) {
        self(frame)
    }
}

pub trait ToFrameHandler {
    fn to_frame_handler(self) -> Box<FrameHandler>;
}

pub trait ToFrameHandlerMut {
    fn to_frame_handler_mut(self) -> Box<FrameHandlerMut>;
}

impl<T> ToFrameHandler for T
    where T: FrameHandler + 'static
{
    fn to_frame_handler(self) -> Box<FrameHandler> {
        Box::new(self) as Box<FrameHandler>
    }
}

impl<T> ToFrameHandlerMut for T
    where T: FrameHandlerMut + 'static
{
    fn to_frame_handler_mut(self) -> Box<FrameHandlerMut> {
        Box::new(self) as Box<FrameHandlerMut>
    }
}

pub struct ReceiptHandler<T>
    where T: ToFrameHandler
{
    pub handler: T,
    _marker: PhantomData<T>,
}

impl<T> ReceiptHandler<T>
    where T: ToFrameHandler
{
    pub fn new(val: T) -> ReceiptHandler<T> {
        ReceiptHandler {
            handler: val,
            _marker: PhantomData,
        }
    }
}

const GRACE_PERIOD_MULTIPLIER: f32 = 2.0;

pub struct Client {
    pub engine: ProtocolEngine<StompProtocol>
}

impl Client {
    pub fn new() -> Client {
        Client {
            engine: mai::protocol_engine(SessionManager).build(),
        }
    }

    pub fn session<T>(&mut self, host: &str, port: u16, event_handler: T) -> SessionBuilder
        where T: Handler + 'static {
        SessionBuilder::new(self, host, port, Box::new(event_handler) as Box<Handler>)
    }
}

pub struct SessionState {
    next_transaction_id: u32,
    next_subscription_id: u32,
    next_receipt_id: u32,
    rx_heartbeat_ms: Option<u32>,
    tx_heartbeat_ms: Option<u32>,
    rx_heartbeat_timeout: Option<mio::Timeout>,
    pub subscriptions: HashMap<String, Subscription>,
}

impl SessionState {
    pub fn new() -> SessionState {
        SessionState {
            next_transaction_id: 0,
            next_subscription_id: 0,
            next_receipt_id: 0,
            rx_heartbeat_ms: None,
            rx_heartbeat_timeout: None,
            tx_heartbeat_ms: None,
            subscriptions: HashMap::new()
        }
    }
}

pub struct SessionData {
    config: SessionConfig,
    state: SessionState,
    event_handler: Box<Handler>
}

impl SessionData {
    pub fn new(config: SessionConfig, event_handler: Box<Handler>) -> SessionData {
        SessionData {
            config: config,
            state: SessionState::new(),
            event_handler: event_handler,
        }
    }

    pub fn split<'a>(&'a mut self) -> (&'a mut SessionConfig, &'a mut SessionState, &'a mut Box<Handler>) {
        let SessionData {
            ref mut config,
            ref mut state,
            ref mut event_handler
        } = *self;
        (config, state, event_handler)
    }

    pub fn state<'a>(&'a mut self) -> &'a mut SessionState {
        &mut self.state
    }

    pub fn event_handler<'a>(&'a mut self) -> &'a mut Box<Handler> {
        &mut self.event_handler
    }

    pub fn config<'a>(&'a mut self) -> &'a mut SessionConfig {
        &mut self.config
    }
}

pub struct Session<'session, 'stream: 'session> {
    config: &'session mut SessionConfig,
    state: &'session mut SessionState,
    stream: &'session mut StreamHandle<'stream, StompProtocol>
}

pub struct SessionContext<'session, 'context: 'session> {
    pub context: &'session mut Context<'context, StompProtocol>,
}

impl<'session, 'context: 'session> SessionContext<'session, 'context> {
    pub fn new(context: &'session mut Context<'context, StompProtocol>)
               -> Session<'session, 'context> {
        Session { context: context }
    }

    pub fn session_and_handler<'a>(&'a mut self) -> (&'a mut Session<'a, 'context>, &'a mut Box<Handler>) {
        let (mut stream, mut session) = self.context.stream_and_session();
        let (mut config, mut state, mut event_handler) = session.split();
        let session = Session {
            config: config,
            state: state,
            stream: stream
        };
        (session, event_handler)
    }

    fn handle_receipt(&mut self, frame: &mut Frame) {
        let (mut session, mut event_handler) = self.session_and_handler();
        let ReceiptId(ref receipt_id) = frame.headers.get_receipt_id().expect("Received RECEIPT frame without a receipt-id");
        //let (stream, session_data) = self.context.stream_and_session();
        event_handler.on_receipt(session, frame);
    }

    // pub fn outstanding_receipts(&mut self) -> Vec<String> {
    //     self.context.session().receipt_handlers.keys().map(|key| key.to_owned()).collect()
    // }

    fn generate_transaction_id(&mut self) -> u32 {
        let mut state = self.context.session().state();
        let id = state.next_transaction_id;
        state.next_transaction_id += 1;
        id
    }

    pub fn generate_subscription_id(&mut self) -> u32 {
        let mut state = self.context.session().state();
        let id = state.next_subscription_id;
        state.next_subscription_id += 1;
        id
    }

    pub fn generate_receipt_id(&mut self) -> u32 {
        let mut state = self.context.session().state();
        let id = state.next_receipt_id;
        state.next_receipt_id += 1;
        id
    }

    pub fn message<'builder, T: ToFrameBody>(&'builder mut self,
                                             destination: &str,
                                             body_convertible: T)
                                             -> MessageBuilder<'builder, 'session, 'context> {
        let send_frame = Frame::send(destination, body_convertible.to_frame_body());
        MessageBuilder {
            session: self,
            frame: send_frame,
        }
    }

    pub fn subscription<'builder, T>(&'builder mut self,
                                     destination: &str,
                                     handler_convertible: T)
                                     -> SubscriptionBuilder<'builder, 'session, 'context>
        where T: ToMessageHandler
    {
        let message_handler: Box<MessageHandler> = handler_convertible.to_message_handler();
        SubscriptionBuilder {
            session: self,
            destination: destination.to_owned(),
            ack_mode: AckMode::Auto,
            handler: message_handler,
            headers: HeaderList::new(),
        }
    }

    pub fn unsubscribe(&mut self, sub_id: &str) -> Result<()> {
        //let _ = self.context.session().subscriptions.remove(sub_id);
        let unsubscribe_frame = Frame::unsubscribe(sub_id.as_ref());
        self.send(unsubscribe_frame)
    }

    pub fn disconnect(&mut self) -> Result<()> {
        let disconnect_frame = Frame::disconnect();
        self.send(disconnect_frame)
    }

    pub fn begin_transaction<'b>(&'b mut self) -> Result<Transaction<'b, 'session, 'context>> {
        let mut transaction = Transaction::new(self.generate_transaction_id(), self);
        let _ = try!(transaction.begin());
        Ok(transaction)
    }

    pub fn send(&mut self, frame: Frame) -> Result<()> {
        // self.on_before_send_callback.on_frame(&mut frame);
        match self.context.stream().send(CompleteFrame(frame)) {
            Ok(_) => Ok(()),//FIXME: Replace 'Other' below with a more meaningful ErrorKind
            Err(_) => {
                Err(Error::new(Other,
                               "Could not send frame: the connection to the server was lost."))
            }
        }
    }

    pub fn invoke_on_before_receive(&mut self, frame: &mut Frame) {
        let (stream, session_data) = self.context.stream_and_session();
        session_data.event_handler.on_before_receive(frame);
    }

    pub fn dispatch(&mut self, frame: &mut Frame) {
        let (stream, session_data) = self.context.stream_and_session();
        // Check for ERROR frame
        match frame.command.as_ref() {
            "ERROR" => return self.context.session().event_handler.on_error(self, &frame),
            "RECEIPT" => return self.handle_receipt(frame),
            "CONNECTED" => return self.on_connected_frame_received(&frame),
            _ => debug!("Received a frame of type {}", &frame.command), // No operation
        };

        let ack_mode: AckMode;
        let callback_result: AckOrNack;
        {
            // This extra scope is required to free up `frame` and `self.subscriptions`
            // following a borrow.

            // Find the subscription ID on the frame that was received
            let header::Subscription(sub_id) = frame.headers
                                                    .get_subscription()
                                                    .expect("Frame did not contain a \
                                                             subscription header.");


            // Look up the appropriate Subscription object
            let subscription = self.context
                                     .session()
                                     .state()
                                     .subscriptions
                                     .get_mut(sub_id)
                                     .expect("Received a message for an unknown subscription.");

            // Take note of the ack_mode used by this Subscription
            ack_mode = subscription.ack_mode;
            // Invoke the callback in the Subscription, providing the frame
            // Take note of whether this frame should be ACKed or NACKed
            callback_result = (*subscription.handler).on_message(&frame);
        }

        debug!("Executing.");
        match ack_mode {
            AckMode::Auto => {
                debug!("Auto ack, no frame sent.");
            }
            AckMode::Client | AckMode::ClientIndividual => {
                let header::Ack(ack_id) = frame.headers
                                               .get_ack()
                                               .expect("Message did not have an 'ack' header.");
                match callback_result {
                    Ack => self.acknowledge_frame(ack_id),
                    Nack => self.negatively_acknowledge_frame(ack_id),
                }
                .unwrap_or_else(|error| panic!(format!("Could not acknowledge frame: {}", error)));
            } // Client | ...
        }
    }

    fn acknowledge_frame(&mut self, ack_id: &str) -> Result<()> {
        let ack_frame = Frame::ack(ack_id);
        self.send(ack_frame)
    }

    fn negatively_acknowledge_frame(&mut self, ack_id: &str) -> Result<()> {
        let nack_frame = Frame::nack(ack_id);
        self.send(nack_frame)
    }

    pub fn on_stream_ready(&mut self) {
        // Add credentials to the header list if specified
        match self.context.session().config().credentials.clone() { // TODO: Refactor to avoid clone()
            Some(credentials) => {
                debug!("Using provided credentials: login '{}', passcode '{}'",
                       credentials.login,
                       credentials.passcode);
                let mut headers = &mut self.context.session().config().headers;
                headers.push(Header::new("login", &credentials.login));
                headers.push(Header::new("passcode", &credentials.passcode));
            }
            None => debug!("No credentials supplied."),
        }

        let connection::HeartBeat(client_tx_ms, client_rx_ms) = self.context
                                                                    .session()
                                                                    .config()
                                                                    .heartbeat;
        let heart_beat_string = format!("{},{}", client_tx_ms, client_rx_ms);
        debug!("Using heartbeat: {},{}", client_tx_ms, client_rx_ms);
        self.context
            .session()
            .config()
            .headers
            .push(Header::new("heart-beat", heart_beat_string.as_ref()));

        let connect_frame = Frame {
            command: "CONNECT".to_string(),
            headers: self.context.session().config().headers.clone(), /* Cloned to allow this SessionBuilder to be re-used */
            body: Vec::new(),
        };

        let _ = self.send(connect_frame); //TODO: Propagate error
    }

    pub fn on_connected_frame_received(&mut self, connected_frame: &Frame) {
        // The Client's requested tx/rx HeartBeat timeouts
        let connection::HeartBeat(client_tx_ms, client_rx_ms) = self.context
                                                                    .session()
                                                                    .config()
                                                                    .heartbeat;

        // The timeouts the server is willing to provide
        let (server_tx_ms, server_rx_ms): (u32, u32) = match connected_frame.headers
                                                                            .get_heart_beat() {
            Some(header::HeartBeat(tx_ms, rx_ms)) => (tx_ms, rx_ms),
            None => (0, 0),
        };
        // TODO: Extract tx_ms + rx_ms from the frame.
        let (agreed_upon_tx_ms, agreed_upon_rx_ms) = Connection::select_heartbeat(client_tx_ms,
                                                                                  client_rx_ms,
                                                                                  server_tx_ms,
                                                                                  server_rx_ms);
        self.context.session().state().rx_heartbeat_ms = Some((agreed_upon_rx_ms as f32 * GRACE_PERIOD_MULTIPLIER) as u32);
        self.context.session().state().tx_heartbeat_ms = Some(agreed_upon_tx_ms);

        self.register_tx_heartbeat_timeout();
        self.register_rx_heartbeat_timeout();
        // TODO: Create + call on_connected callback

        // self.on_connected_callback.on_frame(new Session)
    }

    pub fn register_tx_heartbeat_timeout(&mut self) {
        let tx_heartbeat_ms = self.context
                                  .session()
                                  .state()
                                  .tx_heartbeat_ms
                                  .expect("Trying to register TX heartbeat timeout but no \
                                           tx_heartbeat_ms was set.");
        if tx_heartbeat_ms <= 0 {
            debug!("Heartbeat transmission ms is {}, no need to register a callback.",
                   tx_heartbeat_ms);
            return;
        }
        let _ = self.context.stream().timeout_ms(Timeout::SendHeartBeat, tx_heartbeat_ms as u64);
    }

    pub fn register_rx_heartbeat_timeout(&mut self) {
        let rx_heartbeat_ms = self.context
                                  .session()
                                  .state()
                                  .rx_heartbeat_ms
                                  .expect("Trying to register RX heartbeat timeout but no \
                                           rx_heartbeat_ms was set.");
        if rx_heartbeat_ms <= 0 {
            debug!("Heartbeat receipt ms is {}, no need to register a callback.",
                   rx_heartbeat_ms);
            return;
        }
        let timeout = self.context
                          .stream()
                          .timeout_ms(Timeout::ReceiveHeartBeat, rx_heartbeat_ms as u64)
                          .ok()
                          .expect("Could not register a timeout to receive a heartbeat.");
        self.context.session().state().rx_heartbeat_timeout = Some(timeout);
    }

    pub fn on_heartbeat(&mut self) {
        debug!("Received HeartBeat");
        self.reset_rx_heartbeat_timeout();
    }

    pub fn reset_rx_heartbeat_timeout(&mut self) {
        debug!("Resetting heartbeat rx timeout");
        self.clear_rx_heartbeat_timeout();
        self.register_rx_heartbeat_timeout();
    }

    pub fn clear_rx_heartbeat_timeout(&mut self) {
        debug!("Clearing existing heartbeat rx timeout");
        self.context.session().state().rx_heartbeat_timeout.map(|timeout| {
            let result = self.context.stream().clear_timeout(timeout);
            debug!("Reset complete -> {}", result);
        });
    }

    pub fn send_heartbeat(&mut self) {
        debug!("Sending heartbeat");
        let _ = match self.context.stream().send(HeartBeat) {
            Ok(_) => Ok(()),//FIXME: Replace 'Other' below with a more meaningful ErrorKind
            Err(_) => Err(Error::new(Other, "Could not send a heartbeat. Connection failed.")),
        }; //TODO: Propagate error
    }
}

// impl <'a> Handler for Session<'a> {
//   type Timeout = Timeout;
//   type Message = ();
//
//   fn timeout(&mut self, event_loop: &mut EventLoop<Session<'a>>, timeout: Timeout) {
//     match timeout {
//       Timeout::SendHeartBeat => self.send_heartbeat(event_loop),
//       Timeout::ReceiveHeartBeat => {
//         debug!("Did not receive a heartbeat in time.");
//       },
//     }
//   }

//   fn readable(&mut self, event_loop: &mut EventLoop<Session<'a>>, _token: Token, _: ReadHint) {
//     debug!("Readable! Buffer size: {}", &mut self.read_buffer.len());
//     debug!("Frame buffer length: {}", &mut self.frame_buffer.len());
//     let bytes_read = match self.connection.tcp_stream.read(self.read_buffer.deref_mut()){
//       Ok(0) => {
//         info!("Read 0 bytes. Connection closed by remote host.");
//         self.reconnect(event_loop);
//         return;
//       },
//       Ok(bytes_read) => bytes_read,
//       Err(error) => {
//         info!("Error while reading: {}", error);
//         self.reconnect(event_loop);
//         return;
//       },
//     };
//     info!("Read {} bytes", bytes_read);
//     self.frame_buffer.append(&self.read_buffer[..bytes_read]);
//     let mut num_frames = 0u32;
//     loop {
//       debug!("Reading from frame buffer");
//       match self.frame_buffer.read_transmission() {
//         Some(HeartBeat) => self.on_heartbeat(event_loop),
//         Some(CompleteFrame(mut frame)) => {
//           debug!("Received frame!:\n{}", frame);
//           self.reset_rx_heartbeat_timeout(event_loop);
//           self.on_before_receive_callback.on_frame(&mut frame);
//           self.dispatch(&mut frame);
//           self.frame_buffer.recycle_frame(frame);
//           num_frames += 1;
//         },
//         Some(ConnectionClosed) => {
//           info!("Connection closed by remote host.");
//           self.reconnect(event_loop);
//         },
//         None => {
//           debug!("Done. Read {} frames.", num_frames);
//           break;
//         }
//       }
//     }
//   }
// }


// fn reconnect(&mut self, event_loop: &mut EventLoop<Session>) {
//   let delay_between_attempts = 3_000u32; //TODO: Make this configurable
//   event_loop.deregister(&self.connection.tcp_stream).ok().expect("Failed to deregister dead tcp connection.");
//   self.clear_rx_heartbeat_timeout(event_loop);
//   self.frame_buffer.reset();
//   loop {
//     match self.session_builder.clone().start() {
//       Ok(session) => {
//         info!("Reconnected successfully!");
//         let subscriptions = mem::replace(&mut self.subscriptions, HashMap::new());
//         mem::replace(self, session);
//         self.subscriptions = subscriptions;
//         event_loop.register(&self.connection.tcp_stream, Token(0)).ok().expect("Couldn't register re-established connection with the event loop.");
//         self.register_rx_heartbeat_timeout(event_loop);
//         self.reset_rx_heartbeat_timeout(event_loop);
//         info!("Resubscribing to {} destinations", self.subscriptions.len());
//         let frames : Vec<Frame> = self.subscriptions
//           .values()
//           .map(|subscription| {
//             info!("Re-subscribing to '{}'", &subscription.destination);
//             let mut subscribe_frame = Frame::subscribe(&subscription.id, &subscription.destination, subscription.ack_mode);
//             subscribe_frame.headers.concat(&mut subscription.headers.clone());
//             subscribe_frame.headers.retain(|header| (*header).get_key() != "receipt"); //TODO: Find a way to clean this up.
//             subscribe_frame
//           }).collect();
//         for subscribe_frame in frames {
//           self.send(subscribe_frame).ok().expect("Couldn't re-subscribe.");
//         }
//         break;
//       },
//       Err(error) => {
//         info!("Failed to reconnect: {:?}, retrying again in {}ms", error, delay_between_attempts);
//       }
//     };
//     debug!("Waiting {}ms before attempting to connect again.", delay_between_attempts);
//     thread::sleep_ms(delay_between_attempts);
//   }
// }
