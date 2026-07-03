use crate::{
    net::codec::{write_string, write_var},
    server::commands::COMMANDS,
};

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

    let total =
        1 + COMMANDS.len() as i32 + COMMANDS.iter().map(|c| c.args.len() as i32).sum::<i32>();
    write_var(&mut packet_data, total)?;

    let mut next_index = 1;
    let mut literals = Vec::with_capacity(COMMANDS.len());
    for cmd in COMMANDS.iter() {
        literals.push(next_index);
        next_index += 1 + cmd.args.len() as i32;
    }

    packet_data.push(Flags::Root as u8);
    write_var(&mut packet_data, COMMANDS.len() as i32)?;
    for &i in &literals {
        write_var(&mut packet_data, i)?;
    }

    for (cmd, &literal) in COMMANDS.iter().zip(literals.iter()) {
        let arg_indices: Vec<i32> = (0..cmd.args.len())
            .map(|i| literal + 1 + i as i32)
            .collect();

        let exec = cmd.args.is_empty();
        packet_data.push(Flags::Literal as u8 | if exec { Flags::Executable as u8 } else { 0u8 });

        write_var(&mut packet_data, arg_indices.len() as i32)?;
        for &i in &arg_indices {
            write_var(&mut packet_data, i)?;
        }

        write_string(&mut packet_data, cmd.name)?;

        for (i, arg) in cmd.args.iter().enumerate() {
            let last = i + 1 == cmd.args.len();
            packet_data
                .push(Flags::Argument as u8 | if last { Flags::Executable as u8 } else { 0u8 });

            write_var(&mut packet_data, if last { 0 } else { 1 })?;

            if !last {
                write_var(&mut packet_data, arg_indices[i + 1])?;
            }

            write_string(&mut packet_data, arg.name)?;
            write_var(&mut packet_data, arg.parser.id())?;
            arg.parser.write_props(&mut packet_data)?
        }
    }

    write_var(&mut packet_data, 0)?;

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
