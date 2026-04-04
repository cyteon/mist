use crate::net::codec::{write_string, write_var};

pub async fn send_known_packs<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
) -> anyhow::Result<()> {
    let mut packet_data =
        vec![crate::net::packet::configuration::clientbound::SELECT_KNOWN_PACKS as u8];

    write_var(&mut packet_data, 1)?;

    write_string(&mut packet_data, "minecraft")?;
    write_string(&mut packet_data, "core")?;
    write_string(&mut packet_data, "1.21")?;

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
