extern crate futures;
#[macro_use]
extern crate tokio_core;

mod channel_writer;
mod channel_reader;

use std::{env, str};
use std::net::SocketAddr;
use std::cell::RefCell;
use std::rc::Rc;

use futures::Future;
use futures::stream::Stream;
use tokio_core::io::Io;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tokio_core::channel::channel;
use channel_writer::channel_writer;
use channel_reader::channel_reader;

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let mut l = Core::new().unwrap();
    let handle = l.handle();

    let socket = TcpListener::bind(&addr, &handle).unwrap();
    let tx_local: Rc<RefCell<Vec<::tokio_core::channel::Sender<_>>>> =
        Rc::new(RefCell::new(Vec::new()));
    let (tx, rx) = channel(&handle).unwrap();
    {
        let tx_local = tx_local.clone();
        handle.spawn(rx.for_each(move |x: Vec<u8>| {
                (*tx_local).borrow_mut().retain(|tx| tx.send(x.clone()).is_ok());
                Ok(())
            })
            .map_err(|e| panic!("error: {}", e)));
    }

    println!("Listening on: {}", addr);
    let done = socket.incoming().for_each(|(socket, addr)| {
        let pair = futures::lazy(|| Ok(socket.split()));
        let main_tx = tx.clone();
        let (tx, rx) = channel(&handle).unwrap();
        (*tx_local).borrow_mut().push(tx);
        let amt = pair.and_then(move |(reader, writer)| {
            println!("Connected {}", addr);
            channel_reader(writer, rx).join(channel_writer(reader, main_tx))
        });

        handle.spawn(amt.then(move |_| {
            println!("Disconnected {}", addr);
            Ok(())
        }));
        Ok(())
    });

    l.run(done).unwrap();
}
