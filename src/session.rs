use std::collections::hash_map::HashMap;
use std::io::BufReader;
use std::io::Write;
use std::io::BufWriter;
use std::io::Result;
use std::io::Error;
use std::io::ErrorKind::{Other};
use std::net::TcpStream;
use std::marker::PhantomData;
use connection::Connection;
use subscription::AckMode;
use subscription::AckMode::{Auto, Client, ClientIndividual};
use subscription::AckOrNack;
use subscription::AckOrNack::{Ack, Nack};
use subscription::{Subscription, MessageHandler, ToMessageHandler};
use frame::Frame;
use frame::ToFrameBody;
use frame::Transmission::{HeartBeat, CompleteFrame, ConnectionClosed};
use header;
use header::HeaderList;
use header::ReceiptId;
use header::StompHeaderSet;
use transaction::Transaction;
use message_builder::MessageBuilder;
use subscription_builder::SubscriptionBuilder;

use mio::{EventLoop, Handler, Token, ReadHint, Timeout};
//use mio::tcp::*;


pub trait FrameHandler {
  fn on_frame(&mut self, &Frame);
}

impl <F> FrameHandler for F where F: FnMut(&Frame) {
  fn on_frame(&mut self, frame: &Frame) {
    self(frame)
  }
}

pub trait ToFrameHandler <'a> {
  fn to_frame_handler(self) -> Box<FrameHandler + 'a>;
}

impl <'a, T: 'a> ToFrameHandler <'a> for T where T: FrameHandler {
  fn to_frame_handler(self) -> Box<FrameHandler + 'a> {
    Box::new(self) as Box<FrameHandler>
  }
}

pub struct ReceiptHandler<'a, T> where T: 'a + ToFrameHandler<'a> {
  pub handler: T,
  _marker: PhantomData<&'a T>
}

impl<'a, T> ReceiptHandler<'a, T> where T: 'a + ToFrameHandler<'a> {
  pub fn new(val: T) -> ReceiptHandler<'a T> {
    let r = ReceiptHandler { handler: val, _marker: PhantomData };
    r
  }
}

const GRACE_PERIOD_MULTIPLIER: f64 = 2.0;

pub struct Session <'a> { 
  pub connection : Connection,
  writer: BufWriter<TcpStream>,
  reader: BufReader<TcpStream>,
  next_transaction_id: u32,
  next_subscription_id: u32,
  next_receipt_id: u32,
  rx_heartbeat_ms: u64,
  rx_heartbeat_timeout: Option<Timeout>,
  tx_heartbeat_ms: u64,
  pub subscriptions: HashMap<String, Subscription <'a>>,
  pub receipt_handlers: HashMap<String, Box<FrameHandler + 'a>>,
  error_callback: Box<FrameHandler + 'a>
}

pub enum StompTimeout {
  SendHeartBeat,
  ReceiveHeartBeat
}

impl <'a> Handler for Session<'a> {
  type Timeout = StompTimeout;
  type Message = ();

  fn timeout(&mut self, event_loop: &mut EventLoop<Session<'a>>, timeout: StompTimeout) {
    match timeout {
      StompTimeout::SendHeartBeat => {
        debug!("Sending modified heartbeat"); // FIXME: Make this a function
        let _ = event_loop.timeout_ms(StompTimeout::SendHeartBeat, self.tx_heartbeat_ms);
        self.writer.write("\n".as_bytes()).ok().expect("Could not send a heartbeat. Connection failed.");
        let _ = self.writer.flush();
      },
      StompTimeout::ReceiveHeartBeat => {
        debug!("Did not receive a heartbeat in time.");
      },
    }
  }

  fn readable(&mut self, event_loop: &mut EventLoop<Session<'a>>, _token: Token, _: ReadHint) {
    debug!("Readable!");
    match Frame::read(&mut self.reader) {
      Ok(HeartBeat) => {
        debug!("Received HeartBeat");
        self.rx_heartbeat_timeout.map(|timeout| event_loop.clear_timeout(timeout));
        self.rx_heartbeat_timeout = Some(event_loop.timeout_ms(StompTimeout::ReceiveHeartBeat, self.rx_heartbeat_ms).unwrap());
      },
      Ok(CompleteFrame(frame)) => {
        debug!("Received frame!:\n{}", frame);
        self.dispatch(frame);
      },
      Ok(ConnectionClosed) => panic!("Connection closed by remote host."),
      Err(_) => panic!("Failed to receive Heartbeat or frame.")
    }
  } 
}

impl <'a> Session <'a> {
  pub fn new(connection: Connection, tx_heartbeat_ms: u32, rx_heartbeat_ms: u32) -> Session<'a> {
    let reading_stream = connection.tcp_stream.try_clone().unwrap();
    let writing_stream = reading_stream.try_clone().unwrap();

    let modified_rx_heartbeat_ms : u32 = ((rx_heartbeat_ms as f64) * GRACE_PERIOD_MULTIPLIER) as u32;

    let reader = BufReader::new(reading_stream);
    let writer = BufWriter::new(writing_stream);

    Session {
      connection: connection,
      reader: reader,
      writer: writer,
      next_transaction_id: 0,
      next_subscription_id: 0,
      next_receipt_id: 0,
      rx_heartbeat_ms: modified_rx_heartbeat_ms as u64,
      rx_heartbeat_timeout: None,
      tx_heartbeat_ms: tx_heartbeat_ms as u64 - 1000, //FIXME: Make this configurable
      subscriptions: HashMap::new(),
      receipt_handlers: HashMap::new(),
      error_callback: Box::new(Session::default_error_callback) as Box<FrameHandler>
    }
  }

  fn default_error_callback(frame : &Frame) {
    error!("ERROR received:\n{}", frame);
  }
  
  pub fn on_error<T: 'a>(&mut self, handler_convertible: T) where T : ToFrameHandler<'a> + 'a {
    let handler = handler_convertible.to_frame_handler();
    self.error_callback = handler;
  }

  fn handle_receipt(&mut self, frame: Frame) {
    match frame.headers.get_receipt_id() {
      Some(ReceiptId(ref receipt_id)) => {
        let mut handler = match self.receipt_handlers.remove(*receipt_id) {
          Some(handler) => {
            debug!("Calling handler for ReceiptId '{}'.", *receipt_id);
            handler
          },
          None => {
            panic!("Received unexpected RECEIPT '{}'", *receipt_id)
          }
        };
        handler.on_frame(&frame);
      },
      None => panic!("Received RECEIPT frame without a receipt-id")
    };
  }

  pub fn outstanding_receipts(&self) -> Vec<&str> {
    self.receipt_handlers.keys().map(|key| key.as_ref()).collect()
  }

  fn generate_transaction_id(&mut self) -> u32 {
    let id = self.next_transaction_id;
    self.next_transaction_id += 1;
    id
  }

  pub fn generate_subscription_id(&mut self) -> u32 {
    let id = self.next_subscription_id;
    self.next_subscription_id += 1;
    id
  }

  pub fn generate_receipt_id(&mut self) -> u32 {
    let id = self.next_receipt_id;
    self.next_receipt_id += 1;
    id
  }

  pub fn message<'b, T: ToFrameBody> (&'b mut self, destination: &str, body_convertible: T) -> MessageBuilder<'b, 'a> {
    let send_frame = Frame::send(destination, body_convertible.to_frame_body());
    MessageBuilder {
     session: self,
     frame: send_frame
    }
  }

  pub fn subscription<'b, 'c: 'a, T>(&'b mut self, destination: &'b str, handler_convertible: T) -> SubscriptionBuilder<'b, 'a, 'c> where T: ToMessageHandler<'c> {
    let message_handler : Box<MessageHandler> = handler_convertible.to_message_handler();
    SubscriptionBuilder{
      session: self,
      destination: destination,
      ack_mode: AckMode::Auto,
      handler: message_handler,
      headers: HeaderList::new()
    }
  }

  pub fn unsubscribe(&mut self, sub_id: &str) -> Result<()> {
     let _ = self.subscriptions.remove(sub_id);
     let unsubscribe_frame = Frame::unsubscribe(sub_id.as_ref());
     self.send(unsubscribe_frame)
  }

  pub fn disconnect(&mut self) -> Result<()> {
    let disconnect_frame = Frame::disconnect();
    self.send(disconnect_frame)
  }

  pub fn begin_transaction<'b>(&'b mut self) -> Result<Transaction<'b, 'a>> {
    let mut transaction = Transaction::new(self.generate_transaction_id(), self);
    let _ = try!(transaction.begin());
    Ok(transaction)
  }

  pub fn send(&mut self, frame: Frame) -> Result<()> {
    match frame.write(&mut self.writer) {
      Ok(_) => Ok(()),//FIXME: Replace 'Other' below with a more meaningful ErrorKind
      Err(_) => Err(Error::new(Other, "Could not send frame: the connection to the server was lost."))
    }
  }

  pub fn dispatch(&mut self, frame: Frame) {
    // Check for ERROR frame
    match frame.command.as_ref() {
       "ERROR" => return self.error_callback.on_frame(&frame),
       "RECEIPT" => return self.handle_receipt(frame),
        _ => {} // No operation
    };
 
    let ack_mode : AckMode;
    let callback_result : AckOrNack; 
    { // This extra scope is required to free up `frame` and `self.subscriptions`
      // following a borrow.

      // Find the subscription ID on the frame that was received
      let header::Subscription(sub_id) = 
        frame.headers
        .get_subscription()
        .expect("Frame did not contain a subscription header.");

      // Look up the appropriate Subscription object
      let subscription = 
         self.subscriptions
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
      Auto => {
        debug!("Auto ack, no frame sent.");
      }
      Client | ClientIndividual => {
        let header::Ack(ack_id) = 
          frame.headers
          .get_ack()
          .expect("Message did not have an 'ack' header.");
        match callback_result {
          Ack =>  self.acknowledge_frame(ack_id),
          Nack => self.negatively_acknowledge_frame(ack_id)
        }.unwrap_or_else(|error|panic!(format!("Could not acknowledge frame: {}", error)));
      } // Client | ...
    }
  } 

  fn acknowledge_frame(&mut self, ack_id: &str) -> Result<()> {
    let ack_frame = Frame::ack(ack_id);
    self.send(ack_frame)
  }

  fn negatively_acknowledge_frame(&mut self, ack_id: &str) -> Result<()>{
    let nack_frame = Frame::nack(ack_id);
    self.send(nack_frame)
  }

  pub fn listen(&mut self) -> Result<()> {
    let mut event_loop : EventLoop<Session<'a>> = EventLoop::new().unwrap();
    let _ = event_loop.register(self.reader.get_ref(), Token(0));
    let _ = event_loop.timeout_ms(StompTimeout::SendHeartBeat, self.tx_heartbeat_ms);
    self.rx_heartbeat_timeout = Some(event_loop.timeout_ms(StompTimeout::ReceiveHeartBeat, self.rx_heartbeat_ms).unwrap());
    event_loop.run(self)
  }
}
