use tokio::io::AsyncReadExt;

use crate::net::codec::read_var;

pub struct LoginStartPacket {
    pub name: String,
    pub uuid: String
}

pub async fn read_login_start<R: AsyncReadExt + Unpin>(stream: &mut R) -> anyhow::Result<LoginStartPacket> {
    let _packet_len = read_var(stream).await?;
    let packet_id = read_var(stream).await?;

    if packet_id != 0x00 {
        anyhow::bail!("Expected login start packet ID 0x00, got {}", packet_id);
    }

    let name_len = read_var(stream).await? as usize;
    let mut name_buf = vec![0u8; name_len];
    stream.read_exact(&mut name_buf).await?;
    let name = String::from_utf8(name_buf)?;

    let uuid_encoded = stream.read_i128().await?;
    let uuid = format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        (uuid_encoded >> 96) & 0xffffffff,
        (uuid_encoded >> 80) & 0xffff,
        (uuid_encoded >> 64) & 0xffff,
        (uuid_encoded >> 48) & 0xffff,
        uuid_encoded & 0xffffffffffff
    );

    Ok(LoginStartPacket {
        name,
        uuid
    })
}