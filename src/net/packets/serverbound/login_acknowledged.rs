use tokio::io::AsyncReadExt;

use crate::net::codec::read_var;

pub async fn read_login_acknowledged<R: AsyncReadExt + Unpin>(stream: &mut R) -> anyhow::Result<()> {
    let _packet_len = read_var(stream).await?;
    let packet_id = read_var(stream).await?;

    if packet_id != 0x03 {
        anyhow::bail!("Expected login acknowledged packet ID 0x03, got {}", packet_id);
    }

    Ok(())
}