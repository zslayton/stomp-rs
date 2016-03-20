use std::cmp::max;
use mio::Token;

pub struct Connection {
    pub ip_address: String,
    pub port: u16,
    pub token: Token,
}

#[derive(Clone, Copy)]
pub struct HeartBeat(pub u32, pub u32);
#[derive(Clone, Copy)]
pub struct Credentials<'a>(pub &'a str, pub &'a str);
#[derive(Clone)]
pub struct OwnedCredentials {
    pub login: String,
    pub passcode: String,
}

impl OwnedCredentials {
    pub fn from<'a>(credentials: Credentials<'a>) -> OwnedCredentials {
        OwnedCredentials {
            login: credentials.0.to_owned(),
            passcode: credentials.1.to_owned(),
        }
    }
}

impl Connection {
    pub fn select_heartbeat(client_tx_ms: u32,
                            client_rx_ms: u32,
                            server_tx_ms: u32,
                            server_rx_ms: u32)
                            -> (u32, u32) {
        let heartbeat_tx_ms: u32;
        let heartbeat_rx_ms: u32;
        if client_tx_ms == 0 || server_rx_ms == 0 {
            heartbeat_tx_ms = 0;
        } else {
            heartbeat_tx_ms = max(client_tx_ms, server_rx_ms);
        }
        if client_rx_ms == 0 || server_tx_ms == 0 {
            heartbeat_rx_ms = 0;
        } else {
            heartbeat_rx_ms = max(client_rx_ms, server_tx_ms);
        }
        (heartbeat_tx_ms, heartbeat_rx_ms)
    }
    // pub fn start_session_with_frame(&mut self, connect_frame: Frame) -> Result<(u32, u32)> {
    //   let mut buffered_writer = BufWriter::new(self.tcp_stream.try_clone().unwrap());
    //   try!(connect_frame.write(&mut buffered_writer));
    //   let connected_frame : Frame;
    //   let mut buffered_reader = BufReader::new(self.tcp_stream.try_clone().unwrap());
    //   loop{
    //     let transmission = try!(Frame::read(&mut buffered_reader));
    //     match transmission {
    //       Transmission::HeartBeat => continue,
    //       Transmission::CompleteFrame(frame) => {
    //         connected_frame = frame;
    //         break;
    //       },
    //       Transmission::ConnectionClosed => return Err(Error::new(ErrorKind::ConnectionAborted, "Connection closed by remote host while waiting for CONNECTED frame."))
    //     }
    //   }
    //   match connected_frame.command.as_ref() {
    //     "CONNECTED" => debug!("Received CONNECTED frame: {}", connected_frame),
    //      _ => return Err(Error::new(ErrorKind::InvalidInput, "Could not connect."))
    //   }
    //   match connected_frame.headers.get_heart_beat() {
    //     Some(header::HeartBeat(tx_ms, rx_ms)) => Ok((tx_ms, rx_ms)),
    //     None => Ok((0, 0))
    //   }
    // }
}
