use std::collections::hashmap::HashMap;
use std::io::IoResult;
use std::io::IoError;
use std::io::InvalidInput;
use std::str::from_utf8;
use frame::Frame;
use connection::Connection;
use headers::Header;
use headers::Subscription;
use headers::HeaderList;
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

  fn send_subscribe_frame(&mut self, topic: &str, subscription_id: uint) -> IoResult<String> {
    let mut header_list : HeaderList = HeaderList::with_capacity(3);
    let topic_header_str = format!("destination:{}", topic);
    header_list.push(Header::from_str(topic_header_str.as_slice()).unwrap());
    let subscription_id_str = format!("stomp-rs/{}", subscription_id);
    let subscription_header_str = format!("id:{}", subscription_id_str);
    header_list.push(Header::from_str(subscription_header_str.as_slice()).unwrap());
    header_list.push(Header::from_str("ack:auto").unwrap());
    let subscribe_frame = Frame {
       command : "SUBSCRIBE".to_string(),
       headers : header_list,
       body : Vec::new() 
    };
    println!("Sending frame:\n{}", subscribe_frame);
    try!(subscribe_frame.write(&mut self.connection.writer));
    Ok(subscription_id_str)
  }

  pub fn send_text(&mut self, topic: &str, body: &str) -> IoResult<()> {
    let mut header_list : HeaderList = HeaderList::with_capacity(3);
    let topic_header_str = format!("destination:{}", topic);
    let content_length_str = format!("content-length:{}", body.len());
    header_list.push(Header::from_str(topic_header_str.as_slice()).unwrap());
    header_list.push(Header::from_str(content_length_str.as_slice()).unwrap());
    header_list.push(Header::from_str("content-type:text/plain").unwrap());
    let send_frame = Frame {
      command : "SEND".to_string(),
      headers : header_list,
      body : Vec::from_slice(body.as_bytes())
    };
    Ok(try!(send_frame.write(&mut self.connection.writer)))
  }
 
  pub fn subscribe(&mut self, topic: &str, callback: fn(Frame)->()) -> IoResult<()> {
    let subscription_id = self.generate_subscription_id();
    let subscription_id_str = try!(self.send_subscribe_frame(topic, subscription_id));
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
