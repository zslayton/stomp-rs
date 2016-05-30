use frame::Transmission;
use codec::Codec;
use session::SessionData;
use session_manager::SessionManager;
use timeout::Timeout;
use mai::Protocol;
use mio::tcp::TcpStream;
use handler::Handler;

pub struct StompProtocol;

impl Protocol for StompProtocol {
    type ByteStream = TcpStream;
    type Frame = Transmission;
    type Codec = Codec;
    type Handler = SessionManager;
    type Timeout = Timeout;
    type Session = SessionData;
}
