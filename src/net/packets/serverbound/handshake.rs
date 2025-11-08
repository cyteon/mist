use tokio::io::AsyncReadExt;

use crate::net::codec::read_var;

pub struct HandshakePacket {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: i32,
}

pub async fn read_handshake<R: AsyncReadExt + Unpin>(stream: &mut R) -> anyhow::Result<HandshakePacket> {
    let _packet_len = read_var(stream).await?;
    let packet_id = read_var(stream).await?;

    if packet_id != 0x00 {
        anyhow::bail!("Expected handshake packet ID 0x00, got {}", packet_id);
    }
    
    let protocol_version = read_var(stream).await? as i32;

    let addr_len = read_var(stream).await? as usize;
    let mut addr_buf = vec![0u8; addr_len];
    stream.read_exact(&mut addr_buf).await?;
    let server_address = String::from_utf8(addr_buf)?;

    let server_port = stream.read_u16().await?;
    let next_state = read_var(stream).await? as i32;

    Ok(HandshakePacket {
        protocol_version,
        server_address,
        server_port,
        next_state,
    })
}