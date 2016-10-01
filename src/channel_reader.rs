use std::io::{self, Write, ErrorKind};
use futures::{Future, Poll, Async};
use futures::stream::Stream;
use tokio_core::channel::Receiver;
use std::mem;

pub struct ChannelReader<W: Write> {
    writer: W,
    buffer: Option<Vec<u8>>,
    receiver: Receiver<Vec<u8>>,
}

pub fn channel_reader<W: Write>(writer: W, receiver: Receiver<Vec<u8>>) -> ChannelReader<W> {
    ChannelReader {
        writer: writer,
        buffer: None,
        receiver: receiver,
    }
}

impl<W: Write> Future for ChannelReader<W> {
    type Item = ();
    type Error = io::Error;
    fn poll(&mut self) -> Poll<(), io::Error> {
        loop {
            let buffer = mem::replace(&mut self.buffer, None);
            if let Some(buffer) = buffer {
                match self.writer.write(&buffer) {
                    Ok(t) => t,
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        self.buffer = Some(buffer);
                        return Ok(Async::NotReady);
                    }
                    Err(e) => return Err(e.into()),
                };
            }
            if let Some(_) = self.buffer {
                self.buffer = None;
            }
            match self.receiver.poll() {
                Ok(Async::Ready(Some(data))) => {
                    match self.writer.write(&data) {
                        Ok(t) => t,
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                            self.buffer = Some(data);
                            return Ok(Async::NotReady);
                        }
                        Err(e) => return Err(e.into()),
                    };
                }
                Ok(Async::Ready(None)) => return Ok(Async::Ready(())),
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(e) => return Err(e),
            };
        }
    }
}