use log;
use std::collections::hashmap::HashMap;
use std::io::IoResult;
use std::io::IoError;
use std::io::InvalidInput;
use std::str::from_utf8;
use frame::Frame;
use connection::Connection;
use subscription::AckMode;
use subscription::Auto;
use subscription::AckOrNack;
use subscription::Client;
use subscription::ClientIndividual;
use headers::Header;
use headers::Subscription;
use headers::Ack;
use headers::Id;
use headers::StompHeaderSet;
use transaction::Transaction;

pub struct Session {
  next_transaction_id: uint,
  next_subscription_id: uint,
  next_receipt_id: uint,
  subscriptions: HashMap<String, ::subscription::Subscription>,
  pub connection : Connection,
  error_callback: fn(Frame) -> ()
}

impl Session {
  pub fn new(conn: Connection) -> Session {
    Session {
      next_transaction_id: 0,
      next_subscription_id: 0,
      next_receipt_id: 0,
      subscriptions: HashMap::with_capacity(1),
      connection: conn,
      error_callback: Session::default_error_callback
    }
  }
 
  fn default_error_callback(frame : Frame) {
    debug!("ERROR frame received:\n{}", frame);
  }

  pub fn on_error(&mut self, callback: fn(Frame)) {
    self.error_callback = callback;
  }

  fn generate_transaction_id(&mut self) -> uint {
    let id = self.next_transaction_id;
    self.next_transaction_id += 1;
    id
  }

  fn generate_subscription_id(&mut self) -> uint {
    let id = self.next_subscription_id;
    self.next_subscription_id += 1;
    id
  }

  fn generate_receipt_id(&mut self) -> uint {
    let id = self.next_receipt_id;
    self.next_subscription_id += 1;
    id
  }

  pub fn send_text(&mut self, topic: &str, body: &str) -> IoResult<()> {
    Ok(try!(self.send_bytes(topic, "text/plain", body.as_bytes())))
  }
 
  pub fn send_bytes(&mut self, topic: &str, mime_type: &str, body: &[u8]) -> IoResult<()> {
    let send_frame = Frame::send(topic, mime_type, body);
    self.send(send_frame)
  }

  pub fn send_bytes_with_receipt(&mut self, topic: &str, mime_type: &str, body: &[u8]) -> IoResult<()> {
    let mut send_frame = Frame::send(topic, mime_type, body);
    let receipt_id = format!("message/{}", self.generate_receipt_id().to_string());
    send_frame.headers.push(
      Header::from_key_value("receipt", receipt_id.as_slice())
    );
    Ok(try!(send_frame.write(&mut self.connection.writer)))
  }

  pub fn subscribe(&mut self, topic: &str, ack_mode: AckMode, callback: fn(Frame)->AckOrNack)-> IoResult<String> {
    let next_id = self.generate_subscription_id();
    let sub = ::subscription::Subscription::new(next_id, topic, ack_mode, callback);
    let subscribe_frame = Frame::subscribe(sub.id.as_slice(), sub.topic.as_slice(), ack_mode);
    debug!("Sending frame:\n{}", subscribe_frame);
    try!(subscribe_frame.write(&mut self.connection.writer));
    debug!("Registering callback for subscription id: {}", sub.id);
    let id_to_return = sub.id.to_string();
    self.subscriptions.insert(sub.id.to_string(), sub);
    Ok(id_to_return)
  }

  pub fn unsubscribe(&mut self, sub_id: &str) -> IoResult<()> {
     let sub = self.subscriptions.pop_equiv(&sub_id);
     let unsubscribe_frame = Frame::unsubscribe(sub_id.as_slice());
     self.send(unsubscribe_frame)
  }

  pub fn disconnect(&mut self) -> IoResult<()> {
    let disconnect_frame = Frame::disconnect();
    self.send(disconnect_frame)
  }

  pub fn begin_transaction<'a>(&'a mut self) -> IoResult<Transaction<'a>> {
    let mut tx = Transaction::new(self.generate_transaction_id(), self);
    let _ = try!(tx.begin());
    Ok(tx)
  }

  pub fn receive(&mut self) -> IoResult<Frame> {
    Ok(try!(Frame::read(&mut self.connection.reader)))
  }

  pub fn send(&mut self, frame: Frame) -> IoResult<()> {
    let _ = try!(frame.write(&mut self.connection.writer));
    Ok(())
  }

  pub fn dispatch(&mut self, frame: Frame) -> () {
    // Check for ERROR frame
    match frame.command.as_slice() {
       "ERROR" => return (self.error_callback)(frame),
       "RECEIPT" => {
          debug!("RECEIPT received:\n{}", frame);
          return;
        }
        _ => {} // No operation
    };

    let sub : &::subscription::Subscription;
    let sub_id = match frame.headers.get_subscription() {
      Some(Subscription(ref s)) => s.to_string(),
      None => { 
        debug!("Error: frame did not contain a subscription header.");
        debug!("Frame: {}", frame);
        return;
      }
    };
    sub = match self.subscriptions.find_equiv(&sub_id.as_slice()) {
      Some(sub) => sub,
      None => {
        debug!("Error: Received message for unknown subscription: {}", sub_id);
        return;
      }
    };
    debug!("Executing.");
    match sub.ack_mode {
      Auto => {
        debug!("Auto ack, no frame sent.");
        let _ = (sub.callback)(frame);
      }
      Client | ClientIndividual => {
        let ack_id = match frame.headers.get_ack() {
          Some(Ack(ack_id)) => ack_id.to_string(),
          _ => {
            debug!("Error: Message did not have an ack header.");
            return;
          }
        };
        match (sub.callback)(frame) {
          ::subscription::Ack => {
            let ack_frame = Frame::ack(ack_id.as_slice());
            //match self.send(ack_frame) {
            match ack_frame.write(&mut self.connection.writer) {
              Err(error) => {
                debug!("Couldn't send ACK: {}", error);
                return;
              },
              _ => debug!("ACK sent.")
            }
          },
          ::subscription::Nack => {
            let nack_frame = Frame::nack(ack_id.as_slice());
            //match self.send(nack_frame) {
            match nack_frame.write(&mut self.connection.writer) {
              Err(error) => {
                debug!("Couldn't send NACK: {}", error);
                return;
              },
              _ => debug!("NACK sent.")
            }
          } // Nack
        } // match
      } // Client | ...
    }
  } 

  pub fn listen(&mut self) {
    loop {
      let frame = match self.receive() {
        Ok(f) => f,
        Err(e) => {
          debug!("Error receiving frame: {}", e);
          continue;
        }
      };
      debug!("Received message, dispatching.");
      self.dispatch(frame);
    }
  }      
}
