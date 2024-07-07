use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

mod channel_reader;
mod channel_writer;

use channel_reader::channel_reader;
use channel_writer::channel_writer;
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
    let tx_local: Arc<RwLock<Vec<mpsc::Sender<Vec<u8>>>>> = Arc::new(RwLock::new(Vec::new()));
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(1024);

    let tx_local_clone = Arc::clone(&tx_local);

    // Spawn a task to handle incoming data and distribute it to all senders
    tokio::spawn(async move {
        while let Some(data) = rx.next().await {
            let mut tx_local = tx_local_clone.write().await;
            tx_local.retain(|sender| sender.clone().try_send(data.clone()).is_ok());
        }
    });

    println!("Listening on: {}", addr);

    while let Ok((socket, addr)) = listener.accept().await {
        let main_tx = tx.clone();
        let (tx, mut handle_rx) = mpsc::channel(1024);
        tx_local.write().await.push(tx);

        let (mut reader, mut writer) = socket.into_split();
        let mut handle_tx = main_tx.clone();

        // Spawn a task to handle each connection
        tokio::spawn(async move {
            let channel_reader = channel_reader(&mut writer, &mut handle_rx);
            let channel_writer = channel_writer(&mut reader, &mut handle_tx);
            pin_mut!(channel_reader);
            pin_mut!(channel_writer);
            select(channel_reader, channel_writer).await;
            println!("Disconnected {}", addr);
        });

        println!("Connected {}", addr);
    }
}
