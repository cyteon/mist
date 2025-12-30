use crate::{net::codec::write_var, types::player::Player};
use byteorder::WriteBytesExt;

pub enum PlayerAction {
	AddPlayer, // 0x01
	// InitializeChat(...) // 0x02
	// UpdateGameMode(i32), // 0x04
	UpdateListed(bool), // 0x08
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
	let mut packet_data = vec![0x44];

	let actions_byte = actions.iter().fold(0u8, |acc, action| acc | action.mask());
	packet_data.push(actions_byte);

	write_var(&mut packet_data, players.len() as i32)?;
	for player in players {
		let uuid = player.uuid.replace("-", "");
		let uuid_bytes = hex::decode(uuid).unwrap();
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
				},

				PlayerAction::UpdateListed(listed) => {
					packet_data.push(if *listed { 1 } else { 0 });
				},
			}
		}
	}

	let mut len_prefix = Vec::with_capacity(5);
    write_var(&mut len_prefix, packet_data.len() as i32)?;

    stream.write_all(&len_prefix).await?;
	stream.write_all(&packet_data).await?;
	stream.flush().await?;

	Ok(())
}