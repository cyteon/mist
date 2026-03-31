use crate::net::codec::{write_var, write_string};

pub async fn send_commands<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::COMMANDS as u8];

    // root + 2 commands
    write_var(&mut packet_data, 3)?;
    
    // root
    packet_data.push(0x00);
    write_var(&mut packet_data, 2)?; // 2 children
    write_var(&mut packet_data, 1)?; // tps
    write_var(&mut packet_data, 2)?; // version

    // command 1: /tps
    packet_data.push(0x05); // literal, executable
    write_var(&mut packet_data, 0)?; // no children
    write_string(&mut packet_data, "tps")?;

    // command 2: /version
    packet_data.push(0x05); // literal, executable
    write_var(&mut packet_data, 0)?; // no children
    write_string(&mut packet_data, "version")?;

    // root index
    write_var(&mut packet_data, 0)?;
    
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}