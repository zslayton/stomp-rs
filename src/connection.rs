use std::io::net::tcp::TcpStream;
use std::io::BufferedReader;
use std::io::Timer;
use std::io::IoResult;
use std::io::IoError;
use std::io::InvalidInput;
use std::time::Duration;
use std::str::from_utf8;
use frame::Frame;
use frame::Transmission;
use frame::Heartbeat;
use frame::CompleteFrame;
use session::Session;
use header::Header;

pub static grace_period_multiplier : f64 = 2.0f64;

pub struct Connection {
  pub ip_address : String,
  pub port: u16,
  sender: Sender<Frame>,
  receiver: Receiver<Frame>
}

impl Connection {

  pub fn new(ip_address: &str, port: u16, tx_heartbeat_ms: uint, rx_heartbeat_ms: uint) -> IoResult<Connection> {
    let reading_stream = try!(TcpStream::connect(ip_address, port));
    let writing_stream = reading_stream.clone();
    let (sender_tx, sender_rx) : (Sender<Frame>, Receiver<Frame>) = channel();
    let (receiver_tx, receiver_rx) : (Sender<Frame>, Receiver<Frame>) = channel();

    let modified_rx_heartbeat_ms : uint = ((rx_heartbeat_ms as f64) * grace_period_multiplier) as uint;
    
    spawn(proc(){
      match modified_rx_heartbeat_ms {
        0 => Connection::receive_loop(receiver_tx, reading_stream),
        _ => Connection::receive_loop_with_heartbeat(receiver_tx, reading_stream, Duration::milliseconds(modified_rx_heartbeat_ms as i64))
      } 
    });
    spawn(proc(){
      match tx_heartbeat_ms {
        0 => Connection::send_loop(sender_rx, writing_stream),
        _ => Connection::send_loop_with_heartbeat(sender_rx, writing_stream, Duration::milliseconds(tx_heartbeat_ms as i64))
      } 
    });

    Ok(Connection {
      ip_address: ip_address.to_string(),
      port: port,
      sender : sender_tx,
      receiver : receiver_rx
    })
  }

  pub fn send(&self, frame: Frame) {
    self.sender.send(frame);
  }

  pub fn receive(&self) -> Frame {
    self.receiver.recv()
  }

  fn send_loop(frames_to_send: Receiver<Frame>, mut tcp_stream: TcpStream){
    loop {
      let frame_to_send = frames_to_send.recv();
      frame_to_send.write(&mut tcp_stream).ok().expect("Couldn't send message!");
    }
  }

  fn send_loop_with_heartbeat(frames_to_send: Receiver<Frame>, mut tcp_stream: TcpStream, heartbeat: Duration){
    let mut timer = Timer::new().unwrap(); 
    loop {
      let timeout = timer.oneshot(heartbeat);
      select! {
        () = timeout.recv() => {
          debug!("Sending heartbeat...");
          tcp_stream.write_char('\n').ok().expect("Failed to send heartbeat.");
        },
        frame_to_send = frames_to_send.recv() => {
          frame_to_send.write(&mut tcp_stream).ok().expect("Couldn't send message!");
        }
      }
    }
  }

   fn receive_loop(frame_recipient: Sender<Frame>, tcp_stream: TcpStream){
    let (trans_tx, trans_rx) : (Sender<Transmission>, Receiver<Transmission>) = channel();
    spawn(proc(){
      Connection::read_loop(trans_tx, tcp_stream); 
    });
    loop {
      match trans_rx.recv() {
        Heartbeat => debug!("Received heartbeat"),
        CompleteFrame(frame) => frame_recipient.send(frame)
      }
    }
  }
 

  fn receive_loop_with_heartbeat(frame_recipient: Sender<Frame>, tcp_stream: TcpStream, heartbeat: Duration){
    let (trans_tx, trans_rx) : (Sender<Transmission>, Receiver<Transmission>) = channel();
    spawn(proc(){
      Connection::read_loop(trans_tx, tcp_stream); 
    });


    let mut timer = Timer::new().unwrap(); 
    loop {
      let timeout = timer.oneshot(heartbeat);
      select! {
        () = timeout.recv() => {
          //fail!("Did not receive a heartbeat from the server within {}" + hearbeat);
          error!("Did not receive expected heartbeat!");
        },
        transmission = trans_rx.recv() => {
          match transmission {
            Heartbeat => debug!("Received heartbeat"),
            CompleteFrame(frame) => frame_recipient.send(frame)
          }
        }
      }
    }
  }
 
  fn read_loop(transmission_listener: Sender<Transmission>, tcp_stream: TcpStream){
    let mut reader : BufferedReader<TcpStream> = BufferedReader::new(tcp_stream);
    loop {
      match Frame::read(&mut reader){
        Ok(transmission) => transmission_listener.send(transmission),
        Err(error) => fail!("Couldn't read from server!: {}", error)
      }
    }
  }


  fn read_connected_frame(&mut self) -> IoResult<Frame> {
    let frame : Frame = self.receive(); 
    match frame.command.as_slice() {
      "CONNECTED" => Ok(frame),
      _ => Err(IoError{
             kind: InvalidInput, 
             desc: "Could not connect.",
             detail: from_utf8(frame.body.as_slice()).map(|err: &str| err.to_string())
           })
    }
  }

  pub fn start_session(mut self, tx_heartbeat_ms: uint, rx_heartbeat_ms: uint) -> IoResult<Session> {
    let connect_frame = Frame::connect(tx_heartbeat_ms, rx_heartbeat_ms);
    //let _ = try!(connect_frame.write(&mut self.sender));
    self.send(connect_frame);
    let frame = try!(self.read_connected_frame());
    debug!("Received CONNECTED frame: {}", frame);
    Ok(Session::new(self))
  }

  pub fn start_session_with_credentials(mut self, login: &str, passcode: &str, tx_heartbeat_ms: uint, rx_heartbeat_ms: uint) -> IoResult<Session> {
    let mut connect_frame = Frame::connect(tx_heartbeat_ms, rx_heartbeat_ms);
    connect_frame.headers.push(
      Header::encode_key_value("login", login)
    );
    connect_frame.headers.push(
      Header::encode_key_value("passcode", passcode)
    );
    //let _ = try!(connect_frame.write(&mut self.sender));
    self.send(connect_frame);
    let frame = try!(self.read_connected_frame());
    debug!("Received CONNECTED frame: {}", frame);
    Ok(Session::new(self))
  }

}
