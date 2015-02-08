stomp-rs ![Travis CI Build Status](https://api.travis-ci.org/zslayton/stomp-rs.png?branch=master)
=====
`stomp-rs` provides a full [STOMP](http://stomp.github.io/stomp-specification-1.2.html) 1.2 client implementation for the [Rust programming language](http://www.rust-lang.org/). This allows programs written in Rust to interact with message queueing services like [ActiveMQ](http://activemq.apache.org/) and [RabbitMQ](http://www.rabbitmq.com/).

- [x] Connect
- [x] Subscribe
- [x] Send
- [x] Acknowledge (Auto/Client/ClientIndividual)
- [x] Transactions
- [x] Receipts
- [x] Disconnect
- [x] Heartbeats

The APIs for `stomp-rs` are not yet stable and are likely to fluctuate before v1.0.

## Examples
### Connect / Subscribe / Send
```rust
extern crate stomp;
use stomp::frame::Frame;
use stomp::header::Header;
use stomp::subscription::AckOrNack::Ack;
use stomp::subscription::AckMode::Client;

fn main() {
  
  let destination = "/topic/messages";
  let acknowledge_mode = Client;
  let mut message_count: u64 = 0;

  let mut session = match stomp::session("127.0.0.1", 61613).start() {
      Ok(session) => session,
      Err(error)  => panic!("Could not connect to the server: {}", error)
   };
  
  session.subscribe(destination, acknowledge_mode, |frame: &Frame| {
    message_count += 1;
    println!("Received message #{}:\n{}", message_count, frame);
    Ack
  });
  
  // Send arbitrary bytes with a specified MIME type
  session.send_bytes(topic, "text/plain", "Animal".as_bytes());
  
  // Send UTF-8 text with an assumed MIME type of 'text/plain'
  session.send_text(topic, "Vegetable");
  session.send_text(topic, "Mineral");
  
  session.listen(); // Loops infinitely, awaiting messages

  session.disconnect();
}
```

### Session Settings
```rust
let mut session = match stomp::session("127.0.0.1", 61613)
  .with(Credentials("sullivan", "m1k4d0"))
  .with(HeartBeat(5000, 2000))
  .with(Header::new("custom-client-id", "hmspna4"))
  .start() {
      Ok(session) => session,
      Err(error)  => panic!("Could not connect to the server: {}", error)
   };
```

### Message Settings
```rust
session.message(destination, "text/plain", "Hypoteneuse".as_bytes())
  .with(Header::new("persistent", "true"))
  .with(SuppressedHeader("content-length")
  .send();
```

### Subscription Settings
```rust
  let id = session.subscription(destination, |frame: &Frame| {
    message_count += 1;
    println!("Received message #{}:\n{}", message_count, frame);
    Ack
  })
  .with(AckMode::Client)
  .with(Header::new("custom-subscription-header", "lozenge"))
  .create();
```

### Transactions
```rust
  let mut tx = match session.begin_transaction() {
    Ok(tx) => tx,
    Err(error) => panic!("Could not begin new transaction: {}", error)
  };

  tx.send_text(topic, "Animal");
  tx.send_text(topic, "Vegetable");
  tx.send_text(topic, "Mineral");

  tx.commit(); // Or tx.abort();
```

### Receipts
```rust
  session.send_text_with_receipt(topic, "Modern Major General");
  debug!("Oustanding Receipt IDs: {}", session.oustanding_receipts());
```

### Handling RECEIPT frames
```rust
let handler = |frame: &Frame| debug!("Received receipt for 'Hypoteneuse'", frame);
session.message(destination, "text/plain", "Hypoteneuse".as_bytes())
  .with(ReceiptHandler(handler))
  .send();
```
NOTE: Due to upstream [Rust issue #20174](https://github.com/rust-lang/rust/issues/20174), the compiler will ICE if you attempt to declare the handler inside the `ReceiptHandler` struct in-line. Until this is fixed, the handler must be defined in advance.

### Handling ERROR frames
```rust
session.on_error(|frame: &Frame| {
  panic!("ERROR frame received:\n{}", frame);
});
```

### Cargo.toml
```toml
[package]

name = "stomp_test"
version = "0.0.1"
authors = ["your_name_here"]

[[bin]]

name = "stomp_test"

[dependencies.stomp]

stomp = "*"
```

keywords: `Stomp`, `Rust`, `rust-lang`, `rustlang`, `cargo`, `ActiveMQ`, `RabbitMQ`, `Message Queue`, `MQ`
