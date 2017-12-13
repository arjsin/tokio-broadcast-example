extern crate futures;
#[macro_use]
extern crate tokio_core;
extern crate tokio_io;

mod channel_writer;
mod channel_reader;

use std::env;
use std::net::SocketAddr;
use std::cell::RefCell;
use std::rc::Rc;

use futures::prelude::*;
use futures::unsync::mpsc;
use futures::future::{loop_fn, Loop};
use tokio_io::AsyncRead;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use channel_writer::ChannelWriter;
use channel_reader::ChannelReader;

fn main() {
    let addr = env::args().nth(1).unwrap_or_else(||"127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let mut l = Core::new().unwrap();
    let handle = l.handle();

    let socket = TcpListener::bind(&addr, &handle).unwrap();
    let tx_local: Rc<RefCell<Vec<mpsc::Sender<_>>>> = Rc::new(RefCell::new(Vec::new()));
    let (tx, rx) = mpsc::channel(1024);
    {
        let tx_local = Rc::clone(&tx_local);
        handle.spawn(rx.for_each(move |data: Vec<u8>| {
            // Here we like to remove closed senders
            let tx_local = Rc::clone(&tx_local);
            loop_fn(0, move |index| if index < (*tx_local).borrow().len() {
                let sender = (*tx_local)
                    .borrow()
                    .get(index)
                    .expect("index out of range")
                    .clone();
                let tx_local = Rc::clone(&tx_local);
                let fut = sender.send(data.clone()).then(move |result| {
                    if result.is_ok() {
                        Ok(Loop::Continue(index + 1))
                    } else {
                        // We remove when sender can send no longer
                        (*tx_local).borrow_mut().swap_remove(index);
                        Ok(Loop::Continue(index))
                    }
                });
                Box::new(fut) as Box<Future<Item = _, Error = _>>
            } else {
                Box::new(futures::future::ok(Loop::Break(()))) as Box<Future<Item = _, Error = _>>
            })
        }));
    }

    println!("Listening on: {}", addr);
    let done = socket.incoming().for_each(|(socket, addr)| {
        let main_tx = tx.clone();
        let (tx, rx) = mpsc::channel(1024);
        (*tx_local).borrow_mut().push(tx);
        let (reader, writer) = socket.split();
        let future = ChannelReader::new(writer, rx).join(ChannelWriter::new(reader, main_tx));

        handle.spawn(future.then(move |_| {
            println!("Disconnected {}", addr);
            Ok(())
        }));
        println!("Connected {}", addr);
        Ok(())
    });

    l.run(done).unwrap();
}
