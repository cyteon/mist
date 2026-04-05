use crate::net::codec::{write_string, write_var};
use crate::net::packet::encode_packet;

pub async fn send_update_tags<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::configuration::clientbound::UPDATE_TAGS as u8];

    write_var(&mut packet_data, 1)?;
    write_string(&mut packet_data, "minecraft:block")?;

    write_var(
        &mut packet_data,
        crate::types::tags::BLOCK_TAGS.len() as i32,
    )?;

    for (name, ids) in crate::types::tags::BLOCK_TAGS.iter() {
        write_string(&mut packet_data, name)?;

        write_var(&mut packet_data, ids.len() as i32)?;

        for id in ids.iter() {
            write_var(&mut packet_data, *id)?;
        }
    }

    let encoded = encode_packet(&packet_data);

    stream.write_all(&encoded).await?;
    stream.flush().await?;

    Ok(())
}
