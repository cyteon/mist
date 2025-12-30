use crate::{net::codec::write_var, types::player::Player};
use byteorder::WriteBytesExt;

pub async fn send_login_success<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, player: &Player) -> anyhow::Result<()> {
    let mut packet_data = vec![0x02];

    let uuid_clean = player.uuid.replace("-", "");
    let uuid_bytes = hex::decode(&uuid_clean)?;
    packet_data.extend_from_slice(&uuid_bytes);

    write_var(&mut packet_data, player.username.len() as i32)?;
    packet_data.extend_from_slice(player.username.as_bytes());

    let mut property_count = 0;

    if player.textures.is_some() {
        property_count += 1
    }

    write_var(&mut packet_data, property_count)?;

    if let Some(textures) = &player.textures {
        let name = b"textures";
        write_var(&mut packet_data, name.len() as i32)?;
        packet_data.extend_from_slice(name);

        let texture_bytes = textures.as_bytes();
        write_var(&mut packet_data, texture_bytes.len() as i32)?;
        packet_data.extend(texture_bytes);

        if let Some(texture_signature) = &player.texture_signature {
            packet_data.write_u8(1)?;

            let signature_bytes = texture_signature.as_bytes();
            write_var(&mut packet_data, signature_bytes.len() as i32)?;
            packet_data.extend_from_slice(signature_bytes);
        } else {
            packet_data.write_u8(0)?;
        }
    }

    let mut len_prefix = Vec::with_capacity(5);
    write_var(&mut len_prefix, packet_data.len() as i32)?;

    stream.write_all(&len_prefix).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}