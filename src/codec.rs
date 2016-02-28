use mai;
use frame::Frame;

struct StompCodec;

/*
 * TODO: Move FrameBuffer logic to this struct.
 * In the short term, do not use ParseState. Retry from scratch.
 * In the long term, add CodecState type to Protocol
 */

impl mai::Codec<Frame> for StompCodec {
    fn new() -> Self {
        StompCodec
    }

    fn encode(&mut self, message: &Frame, buffer: &mut [u8]) -> EncodingResult {

    }

    fn decode(&mut self, buffer: &[u8]) -> DecodingResult<String> {

    }
}
