use tokio::io::AsyncReadExt;

use crate::net::codec::read_var;

pub async fn read_chat_message<R: AsyncReadExt + Unpin>(stream: &mut R) -> anyhow::Result<String> {
    let message_length = read_var(stream).await?;
    let mut message_bytes = vec![0u8; message_length as usize];

    stream.read_exact(&mut message_bytes).await?;

    // there is a lot more data like timestamp, but we ignore that for now

    Ok(String::from_utf8(message_bytes)?)
}