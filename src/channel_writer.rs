use std::io::{self, Read};
use futures::{Future, Poll, Async};
use tokio_core::channel::Sender;

pub struct ChannelWriter<R: Read> {
    reader: R,
    sender: Sender<Vec<u8>>,
}

pub fn channel_writer<R: Read>(reader: R, sender: Sender<Vec<u8>>) -> ChannelWriter<R> {
    ChannelWriter {
        reader: reader,
        sender: sender,
    }
}

impl<R: Read> Future for ChannelWriter<R> {
    type Item = ();
    type Error = io::Error;
    fn poll(&mut self) -> Poll<(), io::Error> {
        let mut buf = Vec::with_capacity(2048);
        unsafe {
            buf.set_len(2048);
        }
        let n = try_nb!(self.reader.read(&mut buf));
        if n != 0 {
            unsafe {
                buf.set_len(n);
            }
            if let Err(_) = self.sender.send(buf) {
                println!("channel wrote {} bytes", n);
                Ok(().into())
            } else {
                Ok(Async::NotReady)
            }
        } else {
            Ok(().into())
        }
    }
}