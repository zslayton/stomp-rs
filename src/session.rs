use std::collections::hashmap::HashMap;
use std::io::IoResult;
use std::io::IoError;
use std::io::InvalidInput;
use std::str::from_utf8;
use frame::Frame;
use connection::Connection;
use headers::Subscription;
use headers::StompHeaderSet;

pub struct Session {
  next_message_id: uint,
  next_subscription_id: uint,
  callbacks : HashMap<String, fn(Frame)->()>,
  connection : Connection,
}

impl Session {
  pub fn new(conn: Connection) -> Session {
    Session {
      next_message_id: 0,
      next_subscription_id: 0,
      callbacks: HashMap::with_capacity(1),
      connection: conn
    }
  }

  fn generate_subscription_id(&mut self) -> uint {
    let id = self.next_subscription_id;
    self.next_subscription_id += 1;
    id
  }

  pub fn send_text(&mut self, topic: &str, body: &str) -> IoResult<()> {
    let send_frame = Frame::send(topic, body.as_bytes());
    Ok(try!(send_frame.write(&mut self.connection.writer)))
  }
 
  pub fn subscribe(&mut self, topic: &str, callback: fn(Frame)->()) -> IoResult<()> {
    let subscription_id = self.generate_subscription_id();
    let subscribe_frame = Frame::subscribe(topic, subscription_id);    
    println!("Sending frame:\n{}", subscribe_frame);
    try!(subscribe_frame.write(&mut self.connection.writer));
    let subscription_id_str = match subscribe_frame.headers.get_subscription() {
      Some(Subscription(s)) => s.to_string(),
      None => unreachable!()
    };
    println!("Registering callback for subscription id: {}", subscription_id_str);
    self.callbacks.insert(subscription_id_str, callback);
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
       let subscription_str = match frame.headers.get_subscription() {
        Some(Subscription(ref s)) => s.to_string(),
        None => { 
          println!("Error: frame did not contain a subscription header.");
          return;
        }
      };

      let callback = match self.callbacks.find_equiv(&(subscription_str.as_slice())) {
        Some(c) => c,
        None => {
          println!("Error: Received message for unknown subscription: {}", subscription_str);
          return;
        }
      };
      println!("Found callback. Executing.");
      (*callback)(frame);
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
      self.dispatch(frame);
    }
  }      
}
