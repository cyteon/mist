use byteorder::{BigEndian, WriteBytesExt};

use crate::{
    net::{
        codec::{write_var, write_string}, 
        packets::serverbound::chat_message::Message
    }, 

    types::player::Player
};

pub async fn send_player_chat_message<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    player: &Player,
    target: &mut Player,
    message: &Message
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::PLAYER_CHAT as u8];

    // sector: header

    let global_message_index = {
        target.chat_index += 1;
        target.chat_index
    };
    write_var(&mut packet_data, global_message_index)?;

    let uuid = player.uuid.replace("-", "");
    let uuid_bytes = hex::decode(uuid).unwrap();
    packet_data.extend_from_slice(&uuid_bytes);

    write_var(&mut packet_data, 0)?; // "index", has no usage description

    packet_data.write_u8(0)?; // no signature

    // sector: body

    write_string(&mut packet_data, &message.content)?;

    packet_data.write_i64::<BigEndian>(message.timestamp)?;
    packet_data.write_i64::<BigEndian>(message.salt)?;

    // sector: signature (prefixed array, max len 20)

    write_var(&mut packet_data, 0)?; // empty array

    // sector: other

    packet_data.write_u8(0)?; // unsigned chat preview

    write_var(&mut packet_data, 0)?; // filter type

    // sector: chat formatting

    write_var(&mut packet_data, 1)?; // chat type
    
    let mut sender_nbt = Vec::<u8>::new(); 
    craftflow_nbt::to_writer(
        &mut sender_nbt, &craftflow_nbt::DynNBT::String(player.username.clone())
    ).expect("Failed to write sender NBT");
    packet_data.extend_from_slice(&sender_nbt);
    
    packet_data.write_u8(0)?; // no target name

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}