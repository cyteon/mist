use tokio::io::AsyncReadExt;

use crate::net::codec::read_var;


pub async fn read_chat_command<R: AsyncReadExt + Unpin>(stream: &mut R) -> anyhow::Result<String> {
    let message_length = read_var(stream).await?;
    let mut message_bytes = vec![0u8; message_length as usize];

    stream.read_exact(&mut message_bytes).await?;
    
    Ok(String::from_utf8(message_bytes)?)
}