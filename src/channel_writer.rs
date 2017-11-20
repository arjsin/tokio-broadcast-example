use std::io::{self, Read, ErrorKind};
use std::mem;
use futures::{Future, Poll, Async};
use futures::unsync::mpsc::Sender;
use futures::sink::{Sink, Send};

pub struct ChannelWriter<R: Read> {
    reader: R,
    sender: Sender<Vec<u8>>,
    send: Option<Send<Sender<Vec<u8>>>>,
}



impl<R: Read> ChannelWriter<R> {
    pub fn new(reader: R, sender: Sender<Vec<u8>>) -> ChannelWriter<R> {
        ChannelWriter {
            reader: reader,
            sender: sender,
            send: None,
        }
    }

    fn send_poll(&mut self, mut send: Send<Sender<Vec<u8>>>) -> Poll<(), io::Error> {
        match send.poll() {
            Ok(Async::NotReady) => {
                self.send = Some(send);
                Ok(Async::NotReady)
            }
            Ok(Async::Ready(_)) => Ok(Async::NotReady),
            Err(_) => Err(io::Error::new(ErrorKind::Other, "channel error")),
        }
    }
}

impl<R: Read> Future for ChannelWriter<R> {
    type Item = ();
    type Error = io::Error;
    fn poll(&mut self) -> Poll<(), io::Error> {
        let send = mem::replace(&mut self.send, None);
        if let Some(send) = send {
            self.send_poll(send)
        } else {
            let mut buf = Vec::with_capacity(20);
            unsafe {
                buf.set_len(20);
            }
            let n = try_nb!(self.reader.read(&mut buf));
            if n != 0 {
                unsafe {
                    buf.set_len(n);
                }
                let sender = self.sender.clone();
                let mut send = sender.send(buf);
                self.send_poll(send)
            } else {
                Ok(().into())
            }
        }
    }
}
