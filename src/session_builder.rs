use session::Session;
use frame::Frame;
use option_setter::OptionSetter;
use std::io::Result;
use connection::{Connection, HeartBeat, Credentials, OwnedCredentials};
use header::{HeaderList, Header};

use session::Client;
use session::SessionData;

use std::net::SocketAddr;
use std::net::IpAddr;
use mio::tcp::{TcpSocket, TcpStream};
use std::str::FromStr;

use handler::Handler;

#[derive(Clone)]
pub struct SessionConfig {
    pub host: String,
    pub port: u16,
    pub credentials: Option<OwnedCredentials>,
    pub heartbeat: HeartBeat,
    pub headers: HeaderList,
}

pub struct SessionBuilder<'a> {
    pub client: &'a mut Client,
    pub config: SessionConfig,
    pub event_handler: Box<Handler>
}

impl<'a> SessionBuilder<'a> {
    pub fn new(client: &'a mut Client, host: &str, port: u16, event_handler: Box<Handler>) -> SessionBuilder<'a> {
        let config = SessionConfig {
            host: host.to_owned(),
            port: port,
            credentials: None,
            heartbeat: HeartBeat(0, 0),
            headers: header_list![
           "host" => host,
           "accept-version" => "1.2",
           "content-length" => "0"
          ],
        };
        SessionBuilder {
            client: client,
            config: config,
            event_handler: event_handler
        }
    }

    #[allow(dead_code)]
    pub fn start<'b, 'c> (mut self) {
        let address = SocketAddr::new(IpAddr::from_str(&self.config.host).unwrap(),
                                      self.config.port);
        let socket = TcpSocket::v4().unwrap();
        let (stream, _complete) = socket.connect(&address).unwrap();

        let data = SessionData::new(self.config, self.event_handler);
        let token = self.client.engine.manage(stream, data);
    }

    #[allow(dead_code)]
    pub fn with<'b, T>(self, option_setter: T) -> SessionBuilder<'a>
        where T: OptionSetter<SessionBuilder<'a>>
    {
        option_setter.set_option(self)
    }
}
