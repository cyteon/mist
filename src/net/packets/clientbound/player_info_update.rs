use crate::{net::codec::write_var, types::player::Player};

pub async fn send_player_info_update<W: tokio::io::AsyncWriteExt + Unpin>(
	stream: &mut W,
	players: Vec<&Player>,
) -> anyhow::Result<()> {
	let mut packet_data = vec![0x44];

    // 0x01 = add player
    let mut val: u32 = 0x01;
	loop {
		let mut temp = (val & 0x7F) as u8;
		val >>= 7;
        
		if val != 0 {
			temp |= 0x80;
		}

		packet_data.push(temp);

		if val == 0 {
			break;
		}
	}
    
    // TODO: finish

	write_var(stream, packet_data.len() as i32).await?;
	stream.write_all(&packet_data).await?;
	stream.flush().await?;

	Ok(())
}