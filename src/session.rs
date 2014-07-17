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
use headers::Subscription;
use headers::Ack;
use headers::Id;
use headers::StompHeaderSet;
use transaction::Transaction;

pub struct Session {
  next_transaction_id: uint,
  next_subscription_id: uint,
  subscriptions: HashMap<String, ::subscription::Subscription>,
  pub connection : Connection,
}

impl Session {
  pub fn new(conn: Connection) -> Session {
    Session {
      next_transaction_id: 0,
      next_subscription_id: 0,
      subscriptions: HashMap::with_capacity(1),
      connection: conn
    }
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

  pub fn send_text(&mut self, topic: &str, body: &str) -> IoResult<()> {
    Ok(try!(self.send_bytes(topic, "text/plain", body.as_bytes())))
  }
 
  pub fn send_bytes(&mut self, topic: &str, mime_type: &str, body: &[u8]) -> IoResult<()> {
    let send_frame = Frame::send(topic, mime_type, body);
    Ok(try!(send_frame.write(&mut self.connection.writer)))
  }

  pub fn subscribe(&mut self, topic: &str, ack_mode: AckMode, callback: fn(Frame)->AckOrNack)-> IoResult<String> {
    let next_id = self.generate_subscription_id();
    let sub = ::subscription::Subscription::new(next_id, topic, ack_mode, callback);
    let subscribe_frame = Frame::subscribe(sub.id.as_slice(), sub.topic.as_slice(), ack_mode);
    println!("Sending frame:\n{}", subscribe_frame);
    try!(subscribe_frame.write(&mut self.connection.writer));
    println!("Registering callback for subscription id: {}", sub.id);
    let id_to_return = sub.id.to_string();
    self.subscriptions.insert(sub.id.to_string(), sub);
    Ok(id_to_return)
  }

  pub fn unsubscribe(&mut self, sub_id: &str) -> IoResult<()> {
     let sub = self.subscriptions.pop_equiv(&sub_id);
     let unsubscribe_frame = Frame::unsubscribe(sub_id.as_slice());
     try!(unsubscribe_frame.write(&mut self.connection.writer));
     Ok(())
  }

  pub fn begin_transaction<'a>(&'a mut self) -> IoResult<Transaction<'a>> {
    let mut tx = Transaction::new(self.generate_transaction_id(), self);
    let _ = try!(tx.begin());
    Ok(tx)
  }

  pub fn receive(&mut self) -> IoResult<Frame> {
    let frame = try!(Frame::read(&mut self.connection.reader));
    match frame.command.as_slice() {
      "MESSAGE" => Ok(frame),
      _ => Err(IoError{
             kind: InvalidInput, 
             desc: "Non-MESSAGE frame received.",
             detail: from_utf8(frame.body.as_slice()).map(|err: &str| err.to_string())
           })
    } 
  }

  pub fn dispatch(&mut self, frame: Frame) -> () {
    let sub : &::subscription::Subscription;
    let sub_id = match frame.headers.get_subscription() {
      Some(Subscription(ref s)) => s.to_string(),
      None => { 
        println!("Error: frame did not contain a subscription header.");
        println!("Frame: {}", frame);
        return;
      }
    };
    sub = match self.subscriptions.find_equiv(&sub_id.as_slice()) {
      Some(sub) => sub,
      None => {
        println!("Error: Received message for unknown subscription: {}", sub_id);
        return;
      }
    };
    println!("Executing.");
    match sub.ack_mode {
      Auto => {
        println!("Auto ack, no frame sent.");
        let _ = (sub.callback)(frame);
      }
      Client | ClientIndividual => {
        let ack_id = match frame.headers.get_ack() {
          Some(Ack(ack_id)) => ack_id.to_string(),
          _ => {
            println!("Error: Message did not have an ack header.");
            return;
          }
        };
        match (sub.callback)(frame) {
          ::subscription::Ack => {
            let ack_frame = Frame::ack(ack_id.as_slice());
            match ack_frame.write(&mut self.connection.writer) {
              Err(error) => {
                println!("Couldn't send ACK: {}", error);
                return;
              },
              _ => println!("ACK sent.")
            }
          },
          ::subscription::Nack => {
            let nack_frame = Frame::nack(ack_id.as_slice());
            match nack_frame.write(&mut self.connection.writer) {
              Err(error) => {
                println!("Couldn't send NACK: {}", error);
                return;
              },
              _ => println!("NACK sent.")
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
          println!("Error receiving frame: {}", e);
          continue;
        }
      };
      println!("Received message, dispatching.");
      self.dispatch(frame);
    }
  }      
}
