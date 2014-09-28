use std::collections::hashmap::HashMap;
use std::io::IoResult;
use frame::Frame;
use connection::Connection;
use subscription::AckMode;
use subscription::Auto;
use subscription::AckOrNack;
use subscription::Ack;
use subscription::Nack;
use subscription::Client;
use subscription::ClientIndividual;
use subscription::Subscription;
use header;
use header::Header;
use header::ReceiptId;
use header::StompHeaderSet;
use transaction::Transaction;

//TODO: Replace the HashMap<String, ReceiptStatus> with a Set when
// Sets finally get `equiv` functionality
enum ReceiptStatus {
  Outstanding
}

pub struct Session {
  next_transaction_id: uint,
  next_subscription_id: uint,
  next_receipt_id: uint,
  subscriptions: HashMap<String, Subscription>,
  outstanding_receipts: HashMap<String, ReceiptStatus>, 
  pub connection : Connection,
  receipt_callback: fn(&Frame) -> (), 
  error_callback: fn(&Frame) -> ()
}

impl Session {
  pub fn new(connection: Connection) -> Session {
    Session {
      next_transaction_id: 0,
      next_subscription_id: 0,
      next_receipt_id: 0,
      subscriptions: HashMap::with_capacity(1),
      outstanding_receipts: HashMap::new(),
      connection: connection,
      receipt_callback: Session::default_receipt_callback,
      error_callback: Session::default_error_callback
    }
  }
 
  fn default_error_callback(frame : &Frame) {
    error!("ERROR received:\n{}", frame);
  }
  
  fn default_receipt_callback(frame : &Frame) {
    info!("RECEIPT received:\n{}", frame);
  }

  pub fn on_error(&mut self, callback: fn(&Frame)) {
    self.error_callback = callback;
  }

  fn handle_receipt(&mut self, frame: Frame) {
    match frame.headers.get_receipt_id() {
      Some(ReceiptId(ref receipt_id)) => {
        match self.outstanding_receipts.pop_equiv(receipt_id) {
          Some(Outstanding) => {
            debug!("Removed ReceiptId '{}' from pending receipts.", *receipt_id)
          },
          None => {
            fail!("Received unexpected RECEIPT '{}'", *receipt_id)
          }
        }
      },
      None => fail!("Received RECEIPT frame without a receipt-id")
    };
    (self.receipt_callback)(&frame);
  }

  pub fn on_receipt(&mut self, callback: fn(&Frame)) {
     self.receipt_callback = callback;
  }

  pub fn outstanding_receipts(&self) -> Vec<&str> {
    self.outstanding_receipts.keys().map(|key| key.as_slice()).collect()
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
    self.next_receipt_id += 1;
    id
  }

  pub fn send_text(&mut self, topic: &str, body: &str) -> IoResult<()> {
    Ok(try!(self.send_bytes(topic, "text/plain", body.as_bytes())))
  }
 
  pub fn send_bytes(&mut self, topic: &str, mime_type: &str, body: &[u8]) -> IoResult<()> {
    let send_frame = Frame::send(topic, mime_type, body);
    Ok(self.send(send_frame))
  }

  pub fn send_text_with_receipt(&mut self, topic: &str, body: &str) -> IoResult<()> {
    Ok(try!(self.send_bytes_with_receipt(topic, "text/plain", body.as_bytes())))
  }

  pub fn send_bytes_with_receipt(&mut self, topic: &str, mime_type: &str, body: &[u8]) -> IoResult<()> {
    let mut send_frame = Frame::send(topic, mime_type, body);
    let receipt_id = format!("message/{}", self.generate_receipt_id().to_string());
    send_frame.headers.push(
      Header::encode_key_value("receipt", receipt_id.as_slice())
    );
    //try!(send_frame.write(&mut self.connection.sender));
    self.send(send_frame);
    self.outstanding_receipts.insert(receipt_id, Outstanding);
    Ok(())
  }

  pub fn subscribe(&mut self, topic: &str, ack_mode: AckMode, callback: fn(&Frame)->AckOrNack)-> IoResult<String> {
    let next_id = self.generate_subscription_id();
    let sub = Subscription::new(next_id, topic, ack_mode, callback);
    let subscribe_frame = Frame::subscribe(sub.id.as_slice(), sub.topic.as_slice(), ack_mode);
    debug!("Sending frame:\n{}", subscribe_frame);
    self.send(subscribe_frame);
    //try!(subscribe_frame.write(&mut self.connection.sender));
    debug!("Registering callback for subscription id: {}", sub.id);
    let id_to_return = sub.id.to_string();
    self.subscriptions.insert(sub.id.to_string(), sub);
    Ok(id_to_return)
  }

  pub fn unsubscribe(&mut self, sub_id: &str) -> IoResult<()> {
     let _ = self.subscriptions.pop_equiv(&sub_id);
     let unsubscribe_frame = Frame::unsubscribe(sub_id.as_slice());
     Ok(self.send(unsubscribe_frame))
  }

  pub fn disconnect(&mut self) -> IoResult<()> {
    let disconnect_frame = Frame::disconnect();
    Ok(self.send(disconnect_frame))
  }

  pub fn begin_transaction<'a>(&'a mut self) -> IoResult<Transaction<'a>> {
    let mut tx = Transaction::new(self.generate_transaction_id(), self);
    let _ = try!(tx.begin());
    Ok(tx)
  }

  pub fn receive_frame(&mut self) -> IoResult<Frame> {
    Ok(self.connection.receive())
  }

  pub fn send(&mut self, frame: Frame) {
    self.connection.send(frame);
  }

  pub fn dispatch(&mut self, frame: Frame) -> () {
    // Check for ERROR frame
    match frame.command.as_slice() {
       "ERROR" => return (self.error_callback)(&frame),
       "RECEIPT" => return self.handle_receipt(frame),
        _ => {} // No operation
    };
 
    let ack_mode : AckMode;
    let callback : fn(&Frame) -> AckOrNack;
    { // This extra scope is required to free up `frame` and `subscription`
      // following a borrow.
      let header::Subscription(sub_id) = 
        frame.headers
        .get_subscription()
        .expect("Frame did not contain a subscription header.");

      let (a, c) = 
         self.subscriptions
         .find_equiv(&sub_id)
         .map(|sub| (sub.ack_mode, sub.callback))
         .expect("Received a message for an unknown subscription.");
      ack_mode = a;
      callback = c;
    }

    debug!("Executing.");
    let callback_result : AckOrNack = (callback)(&frame);
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
        }.unwrap_or_else(|error|fail!(format!("Could not acknowledge frame: {}", error)));
      } // Client | ...
    }
  } 

  fn acknowledge_frame(&mut self, ack_id: &str) -> IoResult<()> {
    let ack_frame = Frame::ack(ack_id);
    Ok(self.send(ack_frame))
  }

  fn negatively_acknowledge_frame(&mut self, ack_id: &str) -> IoResult<()>{
    let nack_frame = Frame::nack(ack_id);
    Ok(self.send(nack_frame))
  }

  pub fn receive(&mut self) -> IoResult<()>{
    let frame = try!(self.receive_frame());
    debug!("Received '{}' frame, dispatching.", frame.command);
    Ok(self.dispatch(frame))
  }

  pub fn listen(&mut self) {
    loop {
      let _ = self.receive(); // TODO: Make match with possible failure
    }
  }      
}
