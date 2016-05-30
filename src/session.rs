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
use subscription::{Subscription, MessageHandler};//, MessageHandler, ToMessageHandler};
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
use mai::StreamHandle;
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

pub struct Client<H> where H: Handler {
    pub engine: ProtocolEngine<StompProtocol<H>>,
}

impl<H> Client<H> where H: Handler + 'static {
    pub fn new() -> Client<H> {
        Client { engine: mai::protocol_engine(SessionManager::new()).build() }
    }

    pub fn session(&mut self, host: &str, port: u16, event_handler: H) -> SessionBuilder<H>
    {
        SessionBuilder::new(self, host, port, event_handler)
    }

    pub fn run(self) {
        let _ = self.engine.run();
    }
}

pub struct SessionState<H> where H: Handler {
    next_transaction_id: u32,
    next_subscription_id: u32,
    next_receipt_id: u32,
    pub pending_disconnect: bool,
    pub rx_heartbeat_ms: Option<u32>,
    pub tx_heartbeat_ms: Option<u32>,
    pub rx_heartbeat_timeout: Option<mio::Timeout>,
    pub tx_heartbeat_timeout: Option<mio::Timeout>,
    pub subscriptions: HashMap<String, Subscription<H>>,
}

impl <H> SessionState<H> where H: Handler {
    pub fn new() -> SessionState<H> {
        SessionState {
            next_transaction_id: 0,
            next_subscription_id: 0,
            next_receipt_id: 0,
            pending_disconnect: false,
            rx_heartbeat_ms: None,
            rx_heartbeat_timeout: None,
            tx_heartbeat_ms: None,
            tx_heartbeat_timeout: None,
            subscriptions: HashMap::new(),
        }
    }
}

pub type SessionEventHandler = Box<Handler + 'static>;

pub struct SessionData<H> where H: Handler {
    config: SessionConfig,
    state: SessionState<H>,
    event_handler: H,
}

impl <H> SessionData<H> where H: Handler {
    pub fn new(config: SessionConfig, event_handler: H) -> SessionData<H> {
        SessionData {
            config: config,
            state: SessionState::new(),
            event_handler: event_handler,
        }
    }

    pub fn split<'a>(&'a mut self)
                     -> (&'a mut SessionConfig,
                         &'a mut SessionState<H>,
                         &'a mut H) {
        let SessionData {
            ref mut config,
            ref mut state,
            ref mut event_handler
        } = *self;
        (config, state, event_handler)
    }

    pub fn state<'a>(&'a mut self) -> &'a mut SessionState<H> {
        &mut self.state
    }

    pub fn event_handler<'a>(&'a mut self) -> &'a mut H {
        &mut self.event_handler
    }

    pub fn config<'a>(&'a mut self) -> &'a mut SessionConfig {
        &mut self.config
    }
}


impl <'session, H> Session<'session, H> where H: Handler {
    pub fn send(&mut self, frame: Frame) -> Result<()> {
        // self.on_before_send_callback.on_frame(&mut frame);
        match self.stream.send(CompleteFrame(frame)) {
            Ok(_) => Ok(()),//FIXME: Replace 'Other' below with a more meaningful ErrorKind
            Err(_) => {
                Err(Error::new(Other,
                               "Could not send frame: the connection to the server was lost."))
            }
        }
    }

    pub fn generate_transaction_id(&mut self) -> u32 {
        let id = self.state.next_transaction_id;
        self.state.next_transaction_id += 1;
        id
    }

    pub fn generate_subscription_id(&mut self) -> u32 {
        let id = self.state.next_subscription_id;
        self.state.next_subscription_id += 1;
        id
    }

    pub fn generate_receipt_id(&mut self) -> u32 {
        let id = self.state.next_receipt_id;
        self.state.next_receipt_id += 1;
        id
    }

    pub fn message<'builder, T: ToFrameBody>(&'builder mut self,
                                             destination: &str,
                                             body_convertible: T)
                                             -> MessageBuilder<'builder, 'session, H> {
        let send_frame = Frame::send(destination, body_convertible.to_frame_body());
        //let (session, _event_handler) = self.session_and_handler()
        MessageBuilder {
            session: self,
            frame: send_frame,
        }
    }

    pub fn subscription<'builder>(&'builder mut self,
                                     destination: &str,
                                     message_handler: MessageHandler<H>
                                 )
                                     -> SubscriptionBuilder<'builder, 'session, H>
    {
        //let message_handler: Box<MessageHandler> = handler_convertible.to_message_handler();
        SubscriptionBuilder {
            session: self,
            destination: destination.to_owned(),
            ack_mode: AckMode::Auto,
            handler: message_handler,
            headers: HeaderList::new(),
        }
    }

    pub fn begin_transaction<'b>(&'b mut self) -> Result<Transaction<'b, 'session, H>> {
        let mut transaction = Transaction::new(self);
        let _ = try!(transaction.begin());
        Ok(transaction)
    }

    pub fn unsubscribe(&mut self, sub_id: &str) -> Result<()> {
        // let _ = self.context.session().subscriptions.remove(sub_id);
        let unsubscribe_frame = Frame::unsubscribe(sub_id.as_ref());
        self.send(unsubscribe_frame)
    }

    pub fn disconnect(&mut self) {
        self.state.pending_disconnect = true;
    }

    // pub fn from<'a, 'b: 'a>(context: &'a mut Context<'b, StompProtocol>)
    //                                -> (Session<'a, 'b>, &'a mut SessionEventHandler) {
    pub fn from<'a, 'b: 'a>(context: &'a mut Context<'b, StompProtocol<H>>)
                                   -> (Session<'a, H>, &'a mut H) {
        //let context: &mut Context<StompProtocol> = &mut self.context;
        let (stream, session_data) = context.stream_and_session();
        let (mut config, mut state, mut event_handler) = session_data.split();
        let session = Session {
            config: config,
            state: state,
            stream: stream,
        };
        (session, event_handler)
    }

    pub fn config(&mut self) -> &mut SessionConfig {
        self.config
    }

    pub fn state(&mut self) -> &mut SessionState<H> {
        self.state
    }

    pub fn stream(&mut self) -> &mut StreamHandle<'session, StompProtocol<H>> {
        &mut self.stream
    }
}

pub struct Session<'session, H: 'session> where H: Handler {
    config: &'session mut SessionConfig,
    state: &'session mut SessionState<H>,
    stream: StreamHandle<'session, StompProtocol<H>>,
}

pub struct SessionContext<'session, 'context: 'session, H: 'context> where H: Handler {
    pub context: &'session mut Context<'context, StompProtocol<H>>,
}

impl<'session, 'context: 'session, H: 'session> SessionContext<'session, 'context, H> where H: Handler {
    pub fn new(context: &'session mut Context<'context, StompProtocol<H>>)
               -> SessionContext<'session, 'context, H> {
        SessionContext { context: context }
    }

    // pub fn outstanding_receipts(&mut self) -> Vec<String> {
    //     self.context.session().receipt_handlers.keys().map(|key| key.to_owned()).collect()
    // }


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
