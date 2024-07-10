use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

mod channel_reader;
mod channel_writer;

use channel_reader::read_channel;
use channel_writer::write_channel;
use futures::channel::mpsc;
use futures::future::select;
use futures::pin_mut;
use futures::stream::StreamExt;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    // Get the address from command line arguments or use a default value
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let listener = TcpListener::bind(&addr).await.unwrap();
    let all_clients_tx: Arc<RwLock<Vec<mpsc::Sender<Vec<u8>>>>> = Arc::new(RwLock::new(Vec::new()));
    let (broadcast_tx, mut broadcast_rx) = mpsc::channel::<Vec<u8>>(1024);

    let all_clients_tx_clone = Arc::clone(&all_clients_tx);

    // Spawn a task to handle incoming data and distribute it to all senders
    tokio::spawn(async move {
        while let Some(data) = broadcast_rx.next().await {
            let mut senders = all_clients_tx_clone.write().await;
            senders.retain(|sender| sender.clone().try_send(data.clone()).is_ok());
        }
    });

    println!("Listening on: {}", addr);

    while let Ok((socket, addr)) = listener.accept().await {
        let mut main_tx = broadcast_tx.clone();
        let (client_tx, mut client_rx) = mpsc::channel(1024);
        all_clients_tx.write().await.push(client_tx);

        let (mut socket_reader, mut socket_writer) = socket.into_split();

        // Spawn a task to handle each connection
        tokio::spawn(async move {
            let read_task = read_channel(&mut socket_writer, &mut client_rx);
            let write_task = write_channel(&mut socket_reader, &mut main_tx);
            pin_mut!(read_task);
            pin_mut!(write_task);
            select(read_task, write_task).await;
            println!("Disconnected {}", addr);
        });

        println!("Connected {}", addr);
    }
}
