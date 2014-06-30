use std::collections::hashmap::HashMap;
use std::io::IoResult;
use std::io::IoError;
use std::io::InvalidInput;
use std::str::from_utf8;
use frame::Frame;
use connection::Connection;
use subscription::Subscription;
use headers::Subscription;
use headers::Id;
use headers::StompHeaderSet;

pub struct Session {
  next_message_id: uint,
  next_subscription_id: uint,
  subscriptions: HashMap<String, ::subscription::Subscription>,
  connection : Connection,
}

impl Session {
  pub fn new(conn: Connection) -> Session {
    Session {
      next_message_id: 0,
      next_subscription_id: 0,
      subscriptions: HashMap::with_capacity(1),
      connection: conn
    }
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

  pub fn subscribe(&mut self, topic: &str, callback: fn(Frame)->()) -> IoResult<String> {
    let next_id = self.generate_subscription_id();
    let sub = ::subscription::Subscription::new(next_id, topic, callback);
    let subscribe_frame = Frame::subscribe(sub.id.as_slice(), sub.topic.as_slice());
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
    println!("Found callback. Executing.");
    (sub.callback)(frame);
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
