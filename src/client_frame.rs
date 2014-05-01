use frame::Frame;

enum ClientFrame {
  Connect(~Frame),
  Send(~Frame),
  Subscribe(~Frame),
  Unsubscribe(~Frame),
  Begin(~Frame),
  Commit(~Frame),
  Abort(~Frame),
  Ack(~Frame),
  Nack(~Frame),
  Disconnect(~Frame),
}

/*

TODO methods for each client frame type.

Our client will never have to read these off the wire,
so we can simply offer methods to construct each type
of frame. The methods should require all of the required
Stomp headers as arguments.

Elsewhere (on a StompSession type?), there will be methods
that use the above methods to construct each frame before
actually writing them to the TCP connection.

It might be possible to get away with not creating any 
separate types for ClientFrames. The user will only 
call methods that create short-lived structures behind
the scenes. ServerFrames, however, will require lots
of type safety.
*/

impl ClientFrame {

}
