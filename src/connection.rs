use std::cmp::max;

pub struct Connection {
    pub ip_address: String,
    pub port: u16,
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
    pub fn select_heartbeat(
        client_tx_ms: u32,
        client_rx_ms: u32,
        server_tx_ms: u32,
        server_rx_ms: u32,
    ) -> (u32, u32) {
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
}
