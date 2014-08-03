stomp-rs ![Travis CI Build Status](https://api.travis-ci.org/zslayton/stomp-rs.png?branch=master)
=====
`stomp-rs` aspires to provide a full [STOMP](http://stomp.github.io/stomp-specification-1.2.html) 1.2 client implementation for the [Rust programming language](http://www.rust-lang.org/). This allows programs written in Rust to interact with message queueing services like [ActiveMQ](http://activemq.apache.org/) and [RabbitMQ](http://www.rabbitmq.com/).

`stomp-rs` is in an alpha state and should not be used in production code.

## Examples
### Connect / Subscribe / Send
```rust
extern crate stomp;
use stomp::frame::Frame;
use stomp::subscription::{Ack, AckOrNack, Client};

fn main() {
  let mut session = match stomp::connect("127.0.0.1", 61613) {
    Ok(session)  => session,
    Err(error) => fail!("Could not connect to the server: {}", error)
  };
  
  fn on_message(frame: Frame) -> AckOrNack {
    println!("Received a message:\n{}", frame);
    Ack
  }
  
  let topic = "/topic/messages";
  session.subscribe(topic, Client, on_message); // 'client' acknowledgement mode
  
  // Send arbitrary bytes with a specified MIME type
  session.send_bytes(topic, "text/plain", "Animal".as_bytes());
  
  // Send UTF-8 text with an assumed MIME type of 'text/plain'
  session.send_text(topic, "Vegetable");
  session.send_text(topic, "Mineral");
  
  session.listen(); // Loops infinitely, awaiting messages
}
```

### Transactions
```rust
  let mut tx = match session.begin_transaction() {
    Ok(tx) => tx,
    Err(error) => fail!("Could not begin new transaction: {}", error)
  };

  tx.send_text(topic, "Animal");
  tx.send_text(topic, "Vegetable");
  tx.send_text(topic, "Mineral");

  tx.commit(); // Or tx.abort();
```

### Receipts
```rust
  session.send_text_with_receipt(topic, "Modern Major General");
  debug!("Oustanding Receipt IDs: {}", session.oustanding_receipts()); // A Vec<&str> of Receipt IDs
```

### Handling RECEIPT frames
```rust
  fn on_receipt(frame: Frame) {
    debug!("RECEIPT frame received:\n{}", frame);
  }

  session.on_receipt(on_receipt);
  session.send_text_with_receipt(topic, "Modern Major General");
```

### Handling ERROR frames
```rust
  fn on_error(frame: Frame) {
    fail!("ERROR frame received:\n{}", frame);
  }

  session.on_error(on_error);
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

git = "https://github.com/zslayton/stomp-rs.git"
```

keywords: `Stomp`, `Rust`, `rust-lang`, `rustlang`, `cargo`, `ActiveMQ`, `RabbitMQ`, `Message Queue`, `MQ`
