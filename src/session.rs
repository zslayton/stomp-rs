use std::thread::Thread;
use std::collections::hash_map::HashMap;
use std::time::Duration;
use std::io::BufferedReader;
use std::io::BufferedWriter;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::io::Timer;
use std::io::IoResult;
use std::io::net::tcp::TcpStream;
use connection::Connection;
use subscription::AckMode;
use subscription::AckMode::{Auto, Client, ClientIndividual};
use subscription::AckOrNack;
use subscription::AckOrNack::{Ack, Nack};
use subscription::Subscription;
use frame::Frame;
use frame::Transmission;
use frame::Transmission::{HeartBeat, CompleteFrame};
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
  pub connection : Connection,
  sender: Sender<Frame>,
  receiver: Receiver<Frame>,
  next_transaction_id: uint,
  next_subscription_id: uint,
  next_receipt_id: uint,
  subscriptions: HashMap<String, Subscription>,
  outstanding_receipts: HashMap<String, ReceiptStatus>, 
  receipt_callback: fn(&Frame) -> (), 
  error_callback: fn(&Frame) -> ()
}

pub static GRACE_PERIOD_MULTIPLIER : f64 = 2.0f64;

impl Session {
  pub fn new(connection: Connection, tx_heartbeat_ms: uint, rx_heartbeat_ms: uint) -> Session {
    let reading_stream = connection.tcp_stream.clone();
    let writing_stream = reading_stream.clone();
    let (sender_tx, sender_rx) : (Sender<Frame>, Receiver<Frame>) = channel();
    let (receiver_tx, receiver_rx) : (Sender<Frame>, Receiver<Frame>) = channel();

    let modified_rx_heartbeat_ms : uint = ((rx_heartbeat_ms as f64) * GRACE_PERIOD_MULTIPLIER) as uint;
    let _ = Thread::spawn(move || {
      match modified_rx_heartbeat_ms {
        0 => Session::receive_loop(receiver_tx, reading_stream),
        _ => Session::receive_loop_with_heartbeat(receiver_tx, reading_stream, Duration::milliseconds(modified_rx_heartbeat_ms as i64))
      } 
    }).detach();
    let _ = Thread::spawn(move || {
      match tx_heartbeat_ms {
        0 => Session::send_loop(sender_rx, writing_stream),
        _ => Session::send_loop_with_heartbeat(sender_rx, writing_stream, Duration::milliseconds(tx_heartbeat_ms as i64))
      } 
    }).detach();

    Session {
      connection: connection,
      sender : sender_tx,
      receiver : receiver_rx,
      next_transaction_id: 0,
      next_subscription_id: 0,
      next_receipt_id: 0,
      subscriptions: HashMap::with_capacity(1),
      outstanding_receipts: HashMap::new(),
      receipt_callback: Session::default_receipt_callback,
      error_callback: Session::default_error_callback
    }
  }
 
 fn send_loop(frames_to_send: Receiver<Frame>, tcp_stream: TcpStream){
    let mut writer : BufferedWriter<TcpStream> = BufferedWriter::new(tcp_stream);
    loop {
      let frame_to_send = frames_to_send.recv().ok().expect("Could not receive the next frame: communication was lost with the receiving thread.");
      frame_to_send.write(&mut writer).ok().expect("Couldn't send message!");
    }
  }

  fn send_loop_with_heartbeat(frames_to_send: Receiver<Frame>, tcp_stream: TcpStream, heartbeat: Duration){
    let mut writer : BufferedWriter<TcpStream> = BufferedWriter::new(tcp_stream);
    let mut timer = Timer::new().unwrap(); 
    loop {
      let timeout = timer.oneshot(heartbeat);
      select! {
        _ = timeout.recv() => {
          debug!("Sending heartbeat...");
          writer.write_char('\n').ok().expect("Failed to send heartbeat.");
          let _ = writer.flush();
        },
        frame_to_send = frames_to_send.recv() => {
          frame_to_send.unwrap().write(&mut writer).ok().expect("Couldn't send message!");
        }
      }
    }
  }

   fn receive_loop(frame_recipient: Sender<Frame>, tcp_stream: TcpStream){
    let (trans_tx, trans_rx) : (Sender<Transmission>, Receiver<Transmission>) = channel();
    let _ = Thread::spawn(move || {
      Session::read_loop(trans_tx, tcp_stream); 
    }).detach();
    loop {
      match trans_rx.recv() {
        Ok(HeartBeat) => debug!("Received heartbeat"),
        Ok(CompleteFrame(frame)) => frame_recipient.send(frame).unwrap(),
        Err(err) => panic!("Could not read Transmission from remote host: {}", err)
      }
    }
  }
 

  fn receive_loop_with_heartbeat(frame_recipient: Sender<Frame>, tcp_stream: TcpStream, heartbeat: Duration){
    let (trans_tx, trans_rx) : (Sender<Transmission>, Receiver<Transmission>) = channel();
    let _ = Thread::spawn(move || {
      Session::read_loop(trans_tx, tcp_stream); 
    }).detach();


    let mut timer = Timer::new().unwrap(); 
    loop {
      let timeout = timer.oneshot(heartbeat);
      select! {
        _ = timeout.recv() => error!("Did not receive expected heartbeat!"),
        transmission = trans_rx.recv() => {
          match transmission {
            Ok(HeartBeat) => debug!("Received heartbeat"),
            Ok(CompleteFrame(frame)) => frame_recipient.send(frame).unwrap(),
            Err(err) => panic!("Could not read Transmission from remote host: {}", err)
          }
        }
      }
    }
  }

  fn read_loop(transmission_listener: Sender<Transmission>, tcp_stream: TcpStream){
    let mut reader : BufferedReader<TcpStream> = BufferedReader::new(tcp_stream);
    loop {
      match Frame::read(&mut reader){
         Ok(transmission) => transmission_listener.send(transmission).unwrap(),
         Err(error) => panic!("Couldn't read from server!: {}", error)
      }
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
        match self.outstanding_receipts.remove(*receipt_id) {
          Some(ReceiptStatus::Outstanding) => {
            debug!("Removed ReceiptId '{}' from pending receipts.", *receipt_id)
          },
          None => {
            panic!("Received unexpected RECEIPT '{}'", *receipt_id)
          }
        }
      },
      None => panic!("Received RECEIPT frame without a receipt-id")
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
    self.outstanding_receipts.insert(receipt_id, ReceiptStatus::Outstanding);
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
     let _ = self.subscriptions.remove(sub_id);
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

  pub fn send(&self, frame: Frame) {
    let _ = self.sender.send(frame);
  }

  pub fn receive(&self) -> Frame {
    self.receiver.recv().unwrap()
  }

  pub fn dispatch(&mut self, frame: Frame) {
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
         .get(sub_id)
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
        }.unwrap_or_else(|error|panic!(format!("Could not acknowledge frame: {}", error)));
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

  pub fn listen(&mut self) {
    loop {
      let frame = self.receive();
      debug!("Received '{}' frame, dispatching.", frame.command);
      self.dispatch(frame)
    }
  }      
}
