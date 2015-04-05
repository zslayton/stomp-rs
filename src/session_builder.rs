use session::Session;
use frame::Frame;
use option_setter::OptionSetter;
use std::io::Result;
use connection::{Connection, HeartBeat, Credentials};
use header::{HeaderList, Header};

pub struct SessionBuilder<'a> {
  pub host: &'a str,
  pub port: u16,
  pub credentials: Option<Credentials<'a>>,
  pub heartbeat: HeartBeat,
  pub headers: HeaderList
}

impl <'a> SessionBuilder <'a> {
  pub fn new(host: &'a str, port: u16) -> SessionBuilder<'a> {
    SessionBuilder {
      host: host,
      port: port,
      credentials: None,
      heartbeat: HeartBeat(0,0),
      headers: header_list![ 
       "host" => host,
       "accept-version" => "1.2",
       "content-length" => "0"
      ] 
    }
  }

  #[allow(dead_code)] 
  pub fn start(mut self) -> Result<Session<'a>> {
    // Add credentials to the header list if specified
    match self.credentials {
      Some(Credentials(ref login, ref passcode)) => {
        debug!("Using provided credentials: login '{}', passcode '{}'", login, passcode);
        self.headers.push(Header::new("login", login));
        self.headers.push(Header::new("passcode", passcode));
      },
      None => debug!("No credentials supplied.")
    }
    
    let HeartBeat(client_tx_ms, client_rx_ms) = self.heartbeat;
    let heart_beat_string = format!("{},{}", client_tx_ms, client_rx_ms);
    debug!("Using heartbeat: {},{}", client_tx_ms, client_rx_ms);
    self.headers.push(Header::new("heart-beat", heart_beat_string.as_ref()));

    let connect_frame = Frame {
      command : "CONNECT".to_string(),
      headers : self.headers,
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
