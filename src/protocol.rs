use frame::Transmission;
use codec::Codec;
use session::SessionData;
use session_manager::SessionManager;
use timeout::Timeout;
use mai::Protocol;
use mio::tcp::TcpStream;
use handler::Handler;
use std::marker::PhantomData;

pub struct StompProtocol<H> where H: Handler {
    phantom_data: PhantomData<H>
}

impl <H> Protocol for StompProtocol<H> where H: Handler {
    type ByteStream = TcpStream;
    type Frame = Transmission;
    type Codec = Codec;
    type Handler = SessionManager<H>;
    type Timeout = Timeout;
    type Session = SessionData<H>;
}

impl <H> StompProtocol<H> where H: Handler {
    fn new() -> StompProtocol<H> {
        StompProtocol {
            phantom_data: PhantomData
        }
    }
}
