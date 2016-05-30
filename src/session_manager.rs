use mai::*;
use protocol::StompProtocol;
use timeout::Timeout;
use frame::{Frame, Transmission};
use session::{self, Session, SessionEventHandler};
use subscription::AckMode;
use subscription::AckOrNack;
use subscription::MessageHandler;
use header::{self, Header, HeartBeat, StompHeaderSet};
use handler;
use connection::{self, Connection};
use std::io;
use std::marker::PhantomData;

pub struct SessionManager<H> where H: handler::Handler {
    phantom_data: PhantomData<H>
}

const GRACE_PERIOD_MULTIPLIER: f32 = 2.0;

impl <H> Handler<StompProtocol<H>> for SessionManager<H> where H: handler::Handler {
    fn on_ready(&mut self, context: &mut Context<StompProtocol<H>>) {
        let (mut session, _event_handler) = Session::from(context);
        self.on_stream_ready(&mut session);
    }

    fn on_frame(&mut self, context: &mut Context<StompProtocol<H>>, transmission: Transmission) {
        use frame::Transmission::*;
        let (mut session, mut event_handler) = Session::from(context);
        match transmission {
            HeartBeat => self.on_heartbeat(&mut session),
            CompleteFrame(mut frame) => {
                debug!("Received frame!:\n{}", frame);
                self.reset_rx_heartbeat_timeout(&mut session);
                event_handler.on_before_receive(&mut frame);
                self.dispatch(&mut session, event_handler, &mut frame);
            }
            ConnectionClosed => {
                info!("Connection closed by remote host.");
                // self.reconnect(event_loop);
            }
        }
    }

    fn on_timeout(&mut self, context: &mut Context<StompProtocol<H>>, timeout: Timeout) {
        let (mut session, mut event_handler) = Session::from(context);
        match timeout {
            Timeout::SendHeartBeat => {
                debug!("Sending heartbeat");
                self.send_heartbeat(&mut session);
                self.register_tx_heartbeat_timeout(&mut session);
            }
            Timeout::ReceiveHeartBeat => {
                error!("Did not receive a heartbeat in time."); // TODO: Panic? Handler?
            }
        }
    }

    fn on_error(&mut self, context: &mut Context<StompProtocol<H>>, error: &Error) {
        // This API needs to be fleshed out. Currently only ERROR frames are exposed
        // to the user of the stomp library; IO errors need to be surfaced too.
        panic!("ERROR: {:?}", error);
    }

    fn on_closed(&mut self, context: &mut Context<StompProtocol<H>>) {
        let (mut session, mut event_handler) = Session::from(context);
        info!("Connection closed.");
        self.clear_rx_heartbeat_timeout(&mut session);
        self.clear_tx_heartbeat_timeout(&mut session);
    }
}

impl <H> SessionManager<H> where H: handler::Handler {
    pub fn new() -> SessionManager<H> {
        SessionManager {
            phantom_data: PhantomData
        }
    }

    pub fn dispatch(&mut self, session: &mut Session<H>, event_handler: &mut H, frame: &mut Frame) {
        // Check for ERROR frame
        match frame.command.as_ref() {
            "ERROR" => return event_handler.on_error(session, &frame),
            "RECEIPT" => return self.handle_receipt(session, event_handler, frame), // TODO: Store Frame and pass to on_receipt callback?
            "CONNECTED" => return self.on_connected_frame_received(session, event_handler, &frame),
            _ => debug!("Received a frame of type {}", &frame.command), // No operation
        };

        let ack_mode: AckMode;
        let callback: MessageHandler<H>;
        let callback_result: AckOrNack;
        {
            // This extra scope is required to free up `frame` and `self.subscriptions`
            // following a borrow.

            // Find the subscription ID on the frame that was received
            let header::Subscription(sub_id) = frame.headers
                                                    .get_subscription()
                                                    .expect("Frame did not contain a \
                                                             subscription header.");


             {
                 // Take note of the ack_mode used by this Subscription
                 let subscription = session
                             .state()
                             .subscriptions
                             .get_mut(sub_id)
                             .expect("Received a message for an unknown subscription.");

                 ack_mode = subscription.ack_mode;
                 callback = subscription.handler;
             }


            // Invoke the message callback, providing the frame
            //callback_result = event_handler.on_message(session, frame);
            callback_result = callback(event_handler, session, frame);
        }

        debug!("Executing.");
        match ack_mode {
            AckMode::Auto => {
                debug!("Auto ack, no frame sent.");
            }
            AckMode::Client | AckMode::ClientIndividual => {
                let header::Ack(ack_id) = frame.headers
                                               .get_ack()
                                               .expect("Message did not have an 'ack' header.");
                match callback_result {
                    AckOrNack::Ack => self.acknowledge_frame(session, ack_id),
                    AckOrNack::Nack => self.negatively_acknowledge_frame(session, ack_id),
                }
                .unwrap_or_else(|error| panic!(format!("Could not acknowledge frame: {}", error)));
            } // Client | ...
        }

        // Check to see if any cleanup is necessary
        if session.state().pending_disconnect {
            let _ = session.send(Frame::disconnect());
        }
    }

    fn acknowledge_frame(&mut self, session: &mut Session<H>, ack_id: &str) -> io::Result<()> {
        let ack_frame = Frame::ack(ack_id);
        session.send(ack_frame)
    }

    fn negatively_acknowledge_frame(&mut self, session: &mut Session<H>, ack_id: &str) -> io::Result<()> {
        let nack_frame = Frame::nack(ack_id);
        session.send(nack_frame)
    }

    pub fn on_stream_ready(&mut self, session: &mut Session<H>) {
        // Add credentials to the header list if specified
        match session.config().credentials.clone() { // TODO: Refactor to avoid clone()
            Some(credentials) => {
                debug!("Using provided credentials: login '{}', passcode '{}'",
                       credentials.login,
                       credentials.passcode);
                let mut headers = &mut session.config().headers;
                headers.push(Header::new("login", &credentials.login));
                headers.push(Header::new("passcode", &credentials.passcode));
            }
            None => debug!("No credentials supplied."),
        }

        let connection::HeartBeat(client_tx_ms, client_rx_ms) = session.config().heartbeat;
        let heart_beat_string = format!("{},{}", client_tx_ms, client_rx_ms);
        debug!("Using heartbeat: {},{}", client_tx_ms, client_rx_ms);
        session
            .config()
            .headers
            .push(Header::new("heart-beat", heart_beat_string.as_ref()));

        let connect_frame = Frame {
            command: "CONNECT".to_string(),
            headers: session.config().headers.clone(), /* Cloned to allow this to be re-used */
            body: Vec::new(),
        };

        let _ = session.send(connect_frame); //TODO: Propagate error
    }

    pub fn on_connected_frame_received(&mut self, session: &mut Session<H>, event_handler: &mut H, connected_frame: &Frame) {
        // The Client's requested tx/rx HeartBeat timeouts
        let connection::HeartBeat(client_tx_ms, client_rx_ms) = session.config().heartbeat;

        // The timeouts the server is willing to provide
        let (server_tx_ms, server_rx_ms): (u32, u32) = match connected_frame.headers
                                                                            .get_heart_beat() {
            Some(header::HeartBeat(tx_ms, rx_ms)) => (tx_ms, rx_ms),
            None => (0, 0),
        };

        let (agreed_upon_tx_ms, agreed_upon_rx_ms) = Connection::select_heartbeat(client_tx_ms,
                                                                                  client_rx_ms,
                                                                                  server_tx_ms,
                                                                                  server_rx_ms);
        session.state().rx_heartbeat_ms =
            Some((agreed_upon_rx_ms as f32 * GRACE_PERIOD_MULTIPLIER) as u32);
        session.state().tx_heartbeat_ms = Some(agreed_upon_tx_ms);

        self.register_tx_heartbeat_timeout(session);
        self.register_rx_heartbeat_timeout(session);
        // TODO: Create + call on_connected callback

        event_handler.on_connected(session, connected_frame);
        // self.on_connected_callback.on_frame(new Session)
    }

    fn handle_receipt(&mut self, session: &mut Session<H>, event_handler: &mut H, frame: &mut Frame) {
        let header::ReceiptId(ref receipt_id) = frame
                                             .headers
                                             .get_receipt_id()
                                             .expect("Received RECEIPT frame without a receipt-id");
        if *receipt_id == "msg/disconnect" { //TODO: Make this a reference to a string in the `frame` mod
            return event_handler.on_disconnected(session);
        }
        event_handler.on_receipt(session, frame);
    }

    pub fn register_tx_heartbeat_timeout(&mut self, session: &mut Session<H>) {
        let tx_heartbeat_ms = session
                                .state()
                                .tx_heartbeat_ms
                                .expect("Trying to register TX heartbeat timeout but no \
                                        tx_heartbeat_ms was set.");
        if tx_heartbeat_ms <= 0 {
            debug!("Heartbeat transmission ms is {}, no need to register a callback.",
                   tx_heartbeat_ms);
            return;
        }
        let timeout = session
                .stream()
                .timeout_ms(Timeout::SendHeartBeat, tx_heartbeat_ms as u64)
                .ok()
                .expect("Could not register a timeout to send a heartbeat");
        session.state().tx_heartbeat_timeout = Some(timeout);
    }

    pub fn register_rx_heartbeat_timeout(&mut self, session: &mut Session<H>) {
        let rx_heartbeat_ms = session
                                .state()
                                .rx_heartbeat_ms
                                .unwrap_or_else(|| {
                                    debug!("Trying to register RX heartbeat timeout but no \
                                            rx_heartbeat_ms was set. This is expected for receipt of CONNECTED.");
                                    0
                                });
        if rx_heartbeat_ms <= 0 {
            debug!("Heartbeat receipt ms is {}, no need to register a callback.",
                   rx_heartbeat_ms);
            return;
        }
        let timeout = session
                          .stream()
                          .timeout_ms(Timeout::ReceiveHeartBeat, rx_heartbeat_ms as u64)
                          .ok()
                          .expect("Could not register a timeout to receive a heartbeat.");
        session.state().rx_heartbeat_timeout = Some(timeout);
    }

    pub fn on_heartbeat(&mut self, session: &mut Session<H>) {
        debug!("Received HeartBeat");
        self.reset_rx_heartbeat_timeout(session);
    }

    pub fn reset_rx_heartbeat_timeout(&mut self, session: &mut Session<H>) {
        debug!("Resetting heartbeat rx timeout");
        self.clear_rx_heartbeat_timeout(session);
        self.register_rx_heartbeat_timeout(session);
    }

    pub fn clear_rx_heartbeat_timeout(&mut self, session: &mut Session<H>) {
        debug!("Clearing existing heartbeat rx timeout");
        session.state().rx_heartbeat_timeout.map(|timeout| {
            let result = session.stream().clear_timeout(timeout);
            debug!("Reset complete -> {}", result);
        });
    }

    pub fn clear_tx_heartbeat_timeout(&mut self, session: &mut Session<H>) {
        debug!("Clearing existing heartbeat rx timeout");
        session.state().tx_heartbeat_timeout.map(|timeout| {
            let result = session.stream().clear_timeout(timeout);
            debug!("Reset complete -> {}", result);
        });
    }

    pub fn send_heartbeat(&mut self, session: &mut Session<H>) {
        debug!("Sending heartbeat");
        let _ = match session.stream().send(Transmission::HeartBeat) {
            Ok(_) => Ok(()),//FIXME: Replace 'Other' below with a more meaningful ErrorKind
            Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Could not send a heartbeat. Connection failed.")),
        }; //TODO: Propagate error
    }
}
