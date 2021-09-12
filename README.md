stomp-rs [![](https://api.travis-ci.org/zslayton/stomp-rs.png?branch=master)](https://travis-ci.org/zslayton/stomp-rs) [![](https://img.shields.io/crates/v/stomp.svg)](https://crates.io/crates/stomp)
=====

`stomp-rs` provides a full [STOMP](https://stomp.github.io/stomp-specification-1.2.html) 1.2 client implementation for the [Rust programming language](https://www.rust-lang.org/). This allows programs written in Rust to interact with message queueing services like [ActiveMQ](https://activemq.apache.org/), [RabbitMQ](https://www.rabbitmq.com/), [HornetQ](https://hornetq.jboss.org/) and [OpenMQ](https://javaee.github.io/openmq/).

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
use stomp::subscription::AckOrNack::Ack;

fn main() {
  
  let destination = "/topic/messages";
  let mut message_count: u64 = 0;

  let mut session = match stomp::session("127.0.0.1", 61613).start() {
      Ok(session) => session,
      Err(error)  => panic!("Could not connect to the server: {}", error)
   };
  
  session.subscription(destination, |frame: &Frame| {
    message_count += 1;
    println!("Received message #{}:\n{}", message_count, frame);
    Ack
  }).start();
  
  session.message(destination, "Animal").send();
  session.message(destination, "Vegetable").send();
  session.message(destination, "Mineral").send();
  
  session.listen(); // Loops infinitely, awaiting messages

  session.disconnect();
}
```

### Session Configuration
```rust
use stomp::header::header::Header;
use stomp::connection::{HeartBeat, Credentials};
// ...
let mut session = match stomp::session("127.0.0.1", 61613)
  .with(Credentials("sullivan", "m1k4d0"))
  .with(HeartBeat(5000, 2000))
  .with(Header::new("custom-client-id", "hmspna4"))
  .start() {
      Ok(session) => session,
      Err(error)  => panic!("Could not connect to the server: {}", error)
   };
```

### Message Configuration
```rust
use stomp::header::{Header, SuppressedHeader, ContentType};
// ...
session.message(destination, "Hypoteneuse".as_bytes())
  .with(ContentType("text/plain"))
  .with(Header::new("persistent", "true"))
  .with(SuppressedHeader("content-length")
  .send();
```

### Subscription Configuration
```rust
use stomp::subscription::AckMode;
use stomp::header::Header;
use stomp::frame::Frame;
// ...
  let id = session.subscription(destination, |frame: &Frame| {
    message_count += 1;
    println!("Received message #{}:\n{}", message_count, frame);
    Ack
  })
  .with(AckMode::Client)
  .with(Header::new("custom-subscription-header", "lozenge"))
  .start();
```

### Transactions
```rust
match session.begin_transaction() {
  Ok(mut transaction) => {
    transaction.message(destination, "Animal").send();
    transaction.message(destination, "Vegetable").send();
    transaction.message(destination, "Mineral").send();
    transaction.commit();
},
  Err(error)  => panic!("Could not connect to the server: {}", error)
};
```

### Handling RECEIPT frames
If you include a ReceiptHandler in your message, the client will request that the server send a receipt when it has successfully processed the frame.
```rust
session.message(destination, "text/plain", "Hypoteneuse".as_bytes())
  .with(ReceiptHandler::new(|frame: &Frame| println!("Got a receipt for 'Hypoteneuse'.")))
  .send();
```
### Handling ERROR frames
To handle errors, you can register an error handler
```rust
session.on_error(|frame: &Frame| {
  panic!("ERROR frame received:\n{}", frame);
});
```

### Manipulating inbound and outbound frames
In some cases, brokers impose rules or restrictions which may make it necessary to
directly modify frames in ways that are not conveniently exposed by the API. In such 
cases, you can use the `on_before_send` and `on_before_receive` methods to specify a
callback to perform this custom logic prior to the sending or receipt of each frame.

For example:
```rust
// Require that all NACKs include a header specifying an optional requeue policy
session.on_before_send(|frame: &mut Frame| {
  if frame.command == "NACK" {
    frame.headers.push(Header::new("requeue", "false"));
  }
});

session.on_before_receive(|frame: &mut Frame| {
  if frame.command == "MESSAGE" {
    // Modify the frame
  }
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

keywords: `Stomp`, `Rust`, `rust-lang`, `rustlang`, `cargo`, `ActiveMQ`, `RabbitMQ`, `HornetQ`, `OpenMQ`, `Message Queue`, `MQ`
