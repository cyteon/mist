use crate::{
    log,
    net::codec::{write_string, write_var},
    types::player::Player,
};
use byteorder::WriteBytesExt;

pub enum PlayerAction {
    AddPlayer, // 0x01
    // InitializeChat(...) // 0x02
    UpdateGameMode(i32), // 0x04
    UpdateListed(bool),  // 0x08
                         // UpdateLatency(i32), // 0x10
                         // UpdateDisplayName(Option<...>), // 0x20
                         // UpdateListPriority(i32), // 0x40
                         // UpdateHatStatus(bool), // 0x80
}

impl PlayerAction {
    fn mask(&self) -> u8 {
        match self {
            PlayerAction::AddPlayer => 0x01,
            // PlayerAction::InitializeChat(_) => 0x02,
            PlayerAction::UpdateGameMode(_) => 0x04,
            PlayerAction::UpdateListed(_) => 0x08,
            // PlayerAction::UpdateLatency(_) => 0x10,
            // PlayerAction::UpdateDisplayName(_) => 0x20,
            // PlayerAction::UpdateListPriority(_) => 0x40,
            // PlayerAction::UpdateHatStatus(_) => 0x80,
        }
    }
}

pub async fn send_player_info_update<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    players: Vec<&Player>,
    actions: Vec<PlayerAction>,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::PLAYER_INFO_UPDATE as u8];

    let actions_byte = actions.iter().fold(0u8, |acc, action| acc | action.mask());
    packet_data.push(actions_byte);

    write_var(&mut packet_data, players.len() as i32)?;
    for player in players {
        let uuid = player.uuid.replace("-", "");

        let uuid_bytes = match hex::decode(uuid.clone()) {
            Ok(bytes) => bytes,

            Err(e) => {
                log::log(
                    fancy_log::LogLevel::Error,
                    &format!("Failed to decode UUID {}: {}", uuid, e),
                );

                return Err(anyhow::anyhow!("Failed to decode UUID"));
            }
        };

        packet_data.extend_from_slice(&uuid_bytes);

        for action in &actions {
            match action {
                PlayerAction::AddPlayer => {
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
                }

                PlayerAction::UpdateGameMode(gamemode) => {
                    write_var(&mut packet_data, *gamemode)?;
                }

                PlayerAction::UpdateListed(listed) => {
                    packet_data.push(if *listed { 1 } else { 0 });
                }
            }
        }
    }

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
