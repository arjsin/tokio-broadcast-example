use futures::{channel::mpsc::Sender, SinkExt};
use tokio::io::AsyncReadExt;

pub async fn channel_writer<R: AsyncReadExt + Unpin>(reader: &mut R, tx: &mut Sender<Vec<u8>>) {
    let mut buf = vec![0; 1024];
    while let Ok(n) = reader.read(&mut buf).await {
        if n == 0 {
            break;
        }
        let data = buf[..n].to_vec();
        if tx.send(data).await.is_err() {
            break;
        }
    }
}
