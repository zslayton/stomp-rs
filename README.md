stomp-rs
=====
`stomp-rs` aspires to provide a full [STOMP](http://stomp.github.io/stomp-specification-1.2.html) 1.2 client implementation for the [Rust programming language](http://www.rust-lang.org/). This allows programs written in Rust to interact with message queueing services like [ActiveMQ](http://activemq.apache.org/) and [RabbitMQ](http://www.rabbitmq.com/).

`stomp-rs` is in an alpha state and should not be used in production code.


### Example stomp-rs code
```rust
extern crate stomp;
use stomp::frame::Frame;
use stomp::subscription::Auto; // Acknowledgement mode 'auto'

fn main() {
  let mut session = match stomp::connect("127.0.0.1", 61613) {
    Ok(session)  => session,
    Err(error) => fail!("Could not connect to the server: {}", error)
  };
  
  fn on_message(frame: Frame) -> bool {
    println!("Received a message:\n{}", frame);
    true // Frame handled succesfully. Will send an ACK in Client or ClientIndividual acknowledgement modes
  }
  
  let topic = "/topic/messages";
  session.subscribe(topic, Auto, on_message);
  
  session.send_text(topic, "Animal");
  session.send_text(topic, "Vegetable");
  session.send_text(topic, "Mineral");

  session.listen(); // Loops infinitely, awaiting messages
}
```

### Example Cargo.toml
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
