use tokio::io::AsyncWriteExt;

use crate::{
    net::{
        codec::write_var, 
        packets::serverbound::chat_message::Message
    }, 

    types::player::Player
};

pub async fn send_player_chat_message<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    player: &Player,
    message: &Message
) -> anyhow::Result<()> {
    let mut packet_data = vec![0x3F];

    // sector: header

    let global_message_index = {
        let mut index_lock = crate::GLOBAL_MESSAGE_INDEX.lock().await;
        *index_lock += 1;
        *index_lock
    };
    write_var(&mut packet_data, global_message_index).await?;

    let uuid = player.uuid.replace("-", "");
    let uuid_bytes = hex::decode(uuid).unwrap();
    packet_data.extend_from_slice(&uuid_bytes);

    write_var(&mut packet_data, 0).await?; // "index", has no usage description

    packet_data.write_u8(0).await?; // no signature

    // sector: body

    let message_bytes = message.content.as_bytes();
    write_var(&mut packet_data, message_bytes.len() as i32).await?;
    packet_data.extend_from_slice(message_bytes);

    packet_data.write_i64(message.timestamp).await?;
    packet_data.write_i64(message.salt).await?;

    // sector: signature (prefixed array, max len 20)

    write_var(&mut packet_data, 0).await?; // empty array

    // sector: other

    packet_data.write_u8(0).await?; // unsigned chat preview

    write_var(&mut packet_data, 0).await?; // filter type

    // sector: chat formatting

    write_var(&mut packet_data, 3).await?; // chat type
    
    let mut sender_nbt = Vec::<u8>::new(); 
    craftflow_nbt::to_writer(
        &mut sender_nbt, &craftflow_nbt::DynNBT::String(player.name.clone())
    )
        .expect("Failed to write sender NBT");
    packet_data.extend_from_slice(&sender_nbt);
    
    packet_data.write_u8(0).await?; // no target name

    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}