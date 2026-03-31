use crate::{net::codec::{write_var, write_string}, types::player::Player};
use byteorder::WriteBytesExt;

pub async fn send_login_success<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, player: &Player) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::login::clientbound::LOGIN_FINISHED as u8];
    
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
        write_string(&mut packet_data, "textures")?;
        write_string(&mut packet_data, textures)?;

        if let Some(texture_signature) = &player.texture_signature {
            packet_data.write_u8(1)?;

            write_string(&mut packet_data, texture_signature)?;
        } else {
            packet_data.write_u8(0)?;
        }
    }

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}