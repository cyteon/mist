use std::pin::Pin;

use byteorder::{BigEndian, WriteBytesExt};

use crate::types::colors::RED;
use crate::types::player::Player;

pub mod tps;
pub mod version;

pub type Handler = for<'a> fn(
    &'a [&'a str],
    &'a mut Player,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;

// the mc client will use this for autocomplete and stuff
// todo: finish
#[derive(Clone, Copy)]
pub enum Parser {
    Integer { min: Option<i32>, max: Option<i32> },
    ItemStack,
    Gamemode,
}

impl Parser {
    pub fn id(&self) -> i32 {
        match self {
            Parser::Integer { .. } => 3,
            Parser::ItemStack => 14,
            Parser::Gamemode => 42,
        }
    }

    pub fn write_props(&self, w: &mut Vec<u8>) -> anyhow::Result<()> {
        if let Parser::Integer { min, max } = self {
            let mut flags = 0u8;

            if min.is_some() {
                flags |= 0x01;
            }

            if max.is_some() {
                flags |= 0x02;
            }

            w.push(flags);

            if let Some(min) = min {
                w.write_i32::<BigEndian>(*min)?;
            }

            if let Some(max) = max {
                w.write_i32::<BigEndian>(*max)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct Arg {
    pub name: &'static str,
    pub parser: Parser,
}

pub struct Command {
    pub name: &'static str,
    pub args: &'static [Arg],
    pub handler: Handler,
}

pub static COMMANDS: &[Command] = &[
    Command {
        name: "tps",
        args: &[],
        handler: tps::run,
    },
    Command {
        name: "version",
        args: &[],
        handler: version::run,
    },
];

pub async fn handle_command(command: String, player: &mut Player) -> anyhow::Result<()> {
    let command_parts = command.split_whitespace().collect::<Vec<&str>>();

    for cmd in COMMANDS {
        if cmd.name == command_parts[0] {
            return (cmd.handler)(&command_parts[1..], player).await;
        }
    }

    player
        .send_system_message(format!("{}Unknown command: {}", RED, command_parts[0]))
        .await?;

    Ok(())
}
