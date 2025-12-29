use tokio::io::AsyncReadExt;

use crate::net::codec::read_var;

pub struct Message {
    pub content: String,
    pub timestamp: i64,
    pub salt: i64,
}

pub async fn read_chat_message<R: AsyncReadExt + Unpin>(stream: &mut R) -> anyhow::Result<Message> {
    let message_length = read_var(stream).await?;
    let mut message_bytes = vec![0u8; message_length as usize];

    stream.read_exact(&mut message_bytes).await?;
    let content = String::from_utf8(message_bytes)?;

    let timestamp = stream.read_i64().await?;
    let salt = stream.read_i64().await?;

    Ok(Message {
        content,
        timestamp,
        salt,
    })
}