use crate::{net::codec::write_var, types::player::Player};

pub async fn send_player_info_update<W: tokio::io::AsyncWriteExt + Unpin>(
	stream: &mut W,
	players: Vec<&Player>,
) -> anyhow::Result<()> {
	let mut packet_data = vec![0x44];

	// 0x01 = add player
	write_var(&mut packet_data, 0x01).await?;

	write_var(&mut packet_data, players.len() as i32).await?;
	for player in players {
		let uuid = player.uuid.replace("-", "");
		let uuid_bytes = hex::decode(uuid).unwrap();
		packet_data.extend_from_slice(&uuid_bytes);

		write_var(&mut packet_data, player.name.len() as i32).await?;
		packet_data.extend_from_slice(player.name.as_bytes());
		
		write_var(&mut packet_data, 0).await?; // empty property array
	}

	write_var(stream, packet_data.len() as i32).await?;
	stream.write_all(&packet_data).await?;
	stream.flush().await?;

	Ok(())
}