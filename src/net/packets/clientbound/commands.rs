use crate::net::codec::{write_string, write_var};
use byteorder::{BigEndian, WriteBytesExt};

enum Flags {
    Root = 0x00,
    Literal = 0x01,
    Argument = 0x02,
    Executable = 0x04,
    //HasRedirect    = 0x08,
    //HasSuggestions = 0x10,
    //IsRestricted   = 0x20,
}

pub async fn send_commands<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::COMMANDS as u8];

    // root + 4 commands + 3 args
    write_var(&mut packet_data, 8)?;

    // root
    packet_data.push(Flags::Root as u8);
    write_var(&mut packet_data, 4)?; // 4 commands
    write_var(&mut packet_data, 1)?; // tps
    write_var(&mut packet_data, 2)?; // version
    write_var(&mut packet_data, 3)?; // give
    write_var(&mut packet_data, 6)?; // gamemode

    // command 1: /tps
    packet_data.push(Flags::Literal as u8 | Flags::Executable as u8);
    write_var(&mut packet_data, 0)?; // no children
    write_string(&mut packet_data, "tps")?;

    // command 2: /version
    packet_data.push(Flags::Literal as u8 | Flags::Executable as u8);
    write_var(&mut packet_data, 0)?; // no children
    write_string(&mut packet_data, "version")?;

    // command 3: /give
    packet_data.push(Flags::Literal as u8);
    write_var(&mut packet_data, 1)?;
    write_var(&mut packet_data, 4)?; // item node
    write_string(&mut packet_data, "give")?;

    // /give <item> arg
    packet_data.push(Flags::Argument as u8);
    write_var(&mut packet_data, 1)?;
    write_var(&mut packet_data, 5)?; // amount node
    write_string(&mut packet_data, "item")?;
    write_var(&mut packet_data, 14)?; // minecraft:item_stack parser

    // /give <item> <amount> arg
    packet_data.push(Flags::Argument as u8 | Flags::Executable as u8);
    write_var(&mut packet_data, 0)?;
    write_string(&mut packet_data, "amount")?;
    write_var(&mut packet_data, 3)?; // brigadier:integer parser
    packet_data.push(0x03);
    packet_data.write_i32::<BigEndian>(1)?; // min value
    packet_data.write_i32::<BigEndian>(64)?; // max value

    packet_data.push(Flags::Literal as u8);
    write_var(&mut packet_data, 1)?; // 1 child
    write_var(&mut packet_data, 7)?; // target node
    write_string(&mut packet_data, "gamemode")?;

    packet_data.push(Flags::Argument as u8 | Flags::Executable as u8);
    write_var(&mut packet_data, 0)?; // no children
    write_string(&mut packet_data, "gamemode")?;
    write_var(&mut packet_data, 42)?; // minecraft:gamemode parser

    // root index
    write_var(&mut packet_data, 0)?;

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
