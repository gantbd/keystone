//! Bounded request/reply framing with explicit effect-phase evidence.
//!
//! This crate owns only a four-byte big-endian length prefix, allocation before
//! dispatch, a finite request count and the distinction between input failure
//! and reply loss after dispatch. Consumers retain transport setup, deadlines,
//! codecs, commands, service errors, sessions and retry policy.

use std::io::{self, Read, Write};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameError {
    RequestLimit,
    FrameLimit,
    Capacity,
}

#[derive(Debug)]
pub enum ConnectionError<W, S> {
    /// No consumer command was dispatched.
    BeforeDispatch(io::Error),
    Wire(W),
    Service(S),
    /// A consumer outcome exists, but the complete reply was not written.
    OutcomeUnknown(io::Error),
}

/// Reads, dispatches and replies to exactly one frame.
pub fn serve_one<IO, C, R, W, S>(
    io: &mut IO,
    maximum_frame_bytes: usize,
    mut decode: impl FnMut(&[u8]) -> Result<C, W>,
    mut execute: impl FnMut(C) -> Result<R, S>,
    mut encode: impl FnMut(&R) -> Result<Vec<u8>, W>,
    mut map_frame_error: impl FnMut(FrameError) -> W,
) -> Result<(), ConnectionError<W, S>>
where
    IO: Read + Write,
{
    let mut header = [0_u8; 4];
    io.read_exact(&mut header)
        .map_err(ConnectionError::BeforeDispatch)?;
    serve_frame(
        io,
        header,
        maximum_frame_bytes,
        &mut decode,
        &mut execute,
        &mut encode,
        &mut map_frame_error,
    )
}

/// Serves at most `maximum_requests` frames.
///
/// Clean EOF before a new header closes successfully. Partial headers and
/// bodies are pre-dispatch failures. Once `execute` returns, any write or flush
/// failure is outcome-unknown.
pub fn serve_up_to<IO, C, R, W, S>(
    io: &mut IO,
    maximum_frame_bytes: usize,
    maximum_requests: usize,
    mut decode: impl FnMut(&[u8]) -> Result<C, W>,
    mut execute: impl FnMut(C) -> Result<R, S>,
    mut encode: impl FnMut(&R) -> Result<Vec<u8>, W>,
    mut map_frame_error: impl FnMut(FrameError) -> W,
) -> Result<usize, ConnectionError<W, S>>
where
    IO: Read + Write,
{
    if maximum_requests == 0 {
        return Err(ConnectionError::Wire(map_frame_error(
            FrameError::RequestLimit,
        )));
    }
    let mut served = 0;
    while served < maximum_requests {
        let mut header = [0_u8; 4];
        match io.read(&mut header[..1]) {
            Ok(0) => break,
            Ok(1) => {}
            Ok(_) => unreachable!("one-byte read returned more than one byte"),
            Err(error) => return Err(ConnectionError::BeforeDispatch(error)),
        }
        io.read_exact(&mut header[1..])
            .map_err(ConnectionError::BeforeDispatch)?;
        serve_frame(
            io,
            header,
            maximum_frame_bytes,
            &mut decode,
            &mut execute,
            &mut encode,
            &mut map_frame_error,
        )?;
        served += 1;
    }
    Ok(served)
}

fn serve_frame<IO, C, R, W, S>(
    io: &mut IO,
    header: [u8; 4],
    maximum_frame_bytes: usize,
    decode: &mut impl FnMut(&[u8]) -> Result<C, W>,
    execute: &mut impl FnMut(C) -> Result<R, S>,
    encode: &mut impl FnMut(&R) -> Result<Vec<u8>, W>,
    map_frame_error: &mut impl FnMut(FrameError) -> W,
) -> Result<(), ConnectionError<W, S>>
where
    IO: Read + Write,
{
    let length = usize::try_from(u32::from_be_bytes(header))
        .map_err(|_| ConnectionError::Wire(map_frame_error(FrameError::FrameLimit)))?;
    if length == 0 || length > maximum_frame_bytes {
        return Err(ConnectionError::Wire(map_frame_error(
            FrameError::FrameLimit,
        )));
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| ConnectionError::Wire(map_frame_error(FrameError::Capacity)))?;
    bytes.resize(length, 0);
    io.read_exact(&mut bytes)
        .map_err(ConnectionError::BeforeDispatch)?;
    let command = decode(&bytes).map_err(ConnectionError::Wire)?;
    let reply = execute(command).map_err(ConnectionError::Service)?;
    let encoded = encode(&reply).map_err(ConnectionError::Wire)?;
    let encoded_length = u32::try_from(encoded.len())
        .map_err(|_| ConnectionError::Wire(map_frame_error(FrameError::FrameLimit)))?;
    io.write_all(&encoded_length.to_be_bytes())
        .and_then(|()| io.write_all(&encoded))
        .and_then(|()| io.flush())
        .map_err(ConnectionError::OutcomeUnknown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum WireError {
        Frame(FrameError),
        Codec,
    }

    fn framed(bytes: &[u8]) -> Vec<u8> {
        let mut frame = u32::try_from(bytes.len()).unwrap().to_be_bytes().to_vec();
        frame.extend_from_slice(bytes);
        frame
    }

    #[test]
    fn clean_eof_and_request_bound_are_exact() {
        let mut input = framed(&[1]);
        input.extend_from_slice(&framed(&[2]));
        let mut io = Cursor::new(input);
        let mut seen = Vec::new();
        let served = serve_up_to(
            &mut io,
            1,
            1,
            |bytes| Ok::<_, WireError>(bytes[0]),
            |command| {
                seen.push(command);
                Ok::<_, ()>(command + 1)
            },
            |reply| Ok::<_, WireError>(vec![*reply]),
            WireError::Frame,
        )
        .unwrap();
        assert_eq!(served, 1);
        assert_eq!(seen, [1]);

        let mut empty = Cursor::new(Vec::<u8>::new());
        assert_eq!(
            serve_up_to(
                &mut empty,
                1,
                1,
                |_| Ok::<_, WireError>(()),
                |_| Ok::<_, ()>(()),
                |_| Ok::<_, WireError>(Vec::new()),
                WireError::Frame,
            )
            .unwrap(),
            0,
        );
    }

    #[test]
    fn limits_and_partial_input_never_dispatch() {
        let mut disabled = Cursor::new(Vec::<u8>::new());
        assert!(matches!(
            serve_up_to(
                &mut disabled,
                1,
                0,
                |_| Ok::<_, WireError>(()),
                |_| Ok::<_, ()>(()),
                |_| Ok::<_, WireError>(Vec::new()),
                WireError::Frame,
            ),
            Err(ConnectionError::Wire(WireError::Frame(
                FrameError::RequestLimit
            )))
        ));

        let mut partial = Cursor::new(vec![0, 0, 0, 2, 1]);
        let mut dispatched = false;
        let result = serve_up_to(
            &mut partial,
            2,
            1,
            |_| Ok::<_, WireError>(()),
            |_| {
                dispatched = true;
                Ok::<_, ()>(())
            },
            |_| Ok::<_, WireError>(Vec::new()),
            WireError::Frame,
        );
        assert!(matches!(result, Err(ConnectionError::BeforeDispatch(_))));
        assert!(!dispatched);
    }

    struct FailingWrite {
        input: Cursor<Vec<u8>>,
    }

    impl Read for FailingWrite {
        fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
            self.input.read(bytes)
        }
    }

    impl Write for FailingWrite {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "injected"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn reply_write_failure_is_outcome_unknown_after_one_dispatch() {
        let mut io = FailingWrite {
            input: Cursor::new(framed(&[7])),
        };
        let mut dispatched = 0;
        let result = serve_one(
            &mut io,
            1,
            |bytes| Ok::<_, WireError>(bytes[0]),
            |command| {
                dispatched += 1;
                Ok::<_, ()>(command)
            },
            |reply| Ok::<_, WireError>(vec![*reply]),
            WireError::Frame,
        );
        assert!(matches!(result, Err(ConnectionError::OutcomeUnknown(_))));
        assert_eq!(dispatched, 1);
    }

    #[test]
    fn codec_and_service_failures_retain_their_phase() {
        let mut codec = Cursor::new(framed(&[1]));
        assert!(matches!(
            serve_one(
                &mut codec,
                1,
                |_| Err::<(), _>(WireError::Codec),
                |_| Ok::<_, ()>(()),
                |_| Ok::<_, WireError>(Vec::new()),
                WireError::Frame,
            ),
            Err(ConnectionError::Wire(WireError::Codec))
        ));

        let mut service = Cursor::new(framed(&[1]));
        assert!(matches!(
            serve_one(
                &mut service,
                1,
                |_| Ok::<_, WireError>(()),
                |_| Err::<(), _>("service"),
                |_| Ok::<_, WireError>(Vec::new()),
                WireError::Frame,
            ),
            Err(ConnectionError::Service("service"))
        ));
    }
}
