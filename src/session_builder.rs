use option_setter::OptionSetter;
use connection::{HeartBeat, OwnedCredentials};
use header::{HeaderList, Header};

use session::Client;
use session::SessionData;
use session::SessionEventHandler;

use std::net::SocketAddr;
use std::net::IpAddr;
use mio::tcp::TcpStream;
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

pub struct SessionBuilder<'a, H: 'a> where H: Handler {
    pub client: &'a mut Client<H>,
    pub config: SessionConfig,
    pub event_handler: H,
}

impl<'a, H: 'a> SessionBuilder<'a, H> where H: Handler {
    pub fn new(client: &'a mut Client<H>,
               host: &str,
               port: u16,
               event_handler: H)
               -> SessionBuilder<'a, H> {
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
            event_handler: event_handler,
        }
    }

    #[allow(dead_code)]
    pub fn start<'b, 'c>(mut self) {
        let address = SocketAddr::new(
            IpAddr::from_str(&self.config.host).unwrap(),
            self.config.port
        );
        let stream = TcpStream::connect(&address).expect("Could not create TcpStream.");

        let data = SessionData::new(self.config, self.event_handler);
        let token = self.client.engine.manage(stream, data);
    }

    #[allow(dead_code)]
    pub fn with<'b, T>(self, option_setter: T) -> SessionBuilder<'a, H>
        where T: OptionSetter<SessionBuilder<'a, H>>
    {
        option_setter.set_option(self)
    }
}
