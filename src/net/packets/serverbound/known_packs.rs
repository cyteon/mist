use tokio::io::AsyncReadExt;

use crate::net::codec::read_var;

pub async fn read_known_packs<R: AsyncReadExt + Unpin>(stream: &mut R) -> anyhow::Result<()> {
    let _packet_len = read_var(stream).await?;
    let packet_id = read_var(stream).await?;

    if packet_id != 0x07 {
        anyhow::bail!("Expected known packs packet ID 0x07, got {}", packet_id);
    }

    Ok(())
}