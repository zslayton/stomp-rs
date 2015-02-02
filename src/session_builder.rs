use session::Session;
use frame::Frame;
use option_setter::OptionSetter;
use std::old_io::IoResult;
use connection::{Connection, HeartBeat, Credentials};
use header::{HeaderList, Header};

pub struct SessionBuilder<'a> {
  pub host: &'a str,
  pub port: u16,
  pub credentials: Option<Credentials<'a>>,
  pub heartbeat: HeartBeat,
  pub custom_headers: HeaderList
}

impl <'a> SessionBuilder <'a> {
  pub fn new(host: &'a str, port: u16) -> SessionBuilder<'a> {
    SessionBuilder {
      host: host,
      port: port,
      credentials: None,
      heartbeat: HeartBeat(0,0),
      custom_headers: HeaderList::new()
    }
  }

  #[allow(dead_code)] 
  pub fn start(mut self) -> IoResult<Session<'a>> {
    // Create our base header list with required and known fields
    let mut header_list = header_list![
     "host" => self.host,
     "accept-version" => "1.2",
     "content-length" => "0"
    ];

    // Add credentials to the header list if specified
    match self.credentials {
      Some(Credentials(ref login, ref passcode)) => {
        debug!("Using provided credentials: login '{}', passcode '{}'", login, passcode);
        header_list.push(Header::new("login", login));
        header_list.push(Header::new("passcode", passcode));
      },
      None => debug!("No credentials supplied.")
    }
    
    let HeartBeat(client_tx_ms, client_rx_ms) = self.heartbeat;
    let heart_beat_string = format!("{},{}", client_tx_ms, client_rx_ms);
    debug!("Using heartbeat: {},{}", client_tx_ms, client_rx_ms);
    header_list.push(Header::new("heart-beat", heart_beat_string.as_slice()));

    header_list.concat(&mut self.custom_headers); 

    let connect_frame = Frame {
      command : "CONNECT".to_string(),
      headers : header_list,
      body : Vec::new()
    };

    let mut connection = try!(Connection::new(self.host, self.port));
    let (server_tx_ms, server_rx_ms) = try!(connection.start_session_with_frame(connect_frame));
    let (tx_ms, rx_ms) = Connection::select_heartbeat(
      client_tx_ms,
      client_rx_ms,
      server_tx_ms,
      server_rx_ms
    );

    Ok(Session::new(connection, tx_ms, rx_ms))
  }

  #[allow(dead_code)] 
  pub fn with<T>(self, option_setter: T) -> SessionBuilder<'a> where T: OptionSetter<SessionBuilder<'a>> {
    option_setter.set_option(self) 
  } 
}
