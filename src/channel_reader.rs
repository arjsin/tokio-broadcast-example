use futures::{channel::mpsc::Receiver, StreamExt};
use tokio::io::AsyncWriteExt;

pub async fn read_channel<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    rx: &mut Receiver<Vec<u8>>,
) {
    while let Some(data) = rx.next().await {
        writer.write_all(&data).await.expect("unhandled write error");
    }
}
