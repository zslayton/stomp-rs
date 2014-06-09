stomp-rs
=====
`stomp-rs` aspires to provide a full STOMP 1.2 client implementation for the Rust
programming language. It is in a pre-alpha state and should not be used.

```rust
let session = match Stomp::connect("127.0.0.1", 61613) {
  Ok(s)  => s,
  Err(e) => fail!("Could not connect to the server! {}", e)
};

fn on_message(frame: Frame){
  println!("Received a message:\n{}", frame.to_str());
}

session.subscribe("/topic/messages", on_message);
session.listen(); // Loops infinitely, awaiting messages
```
