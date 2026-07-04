use std::pin::Pin;

use byteorder::{BigEndian, WriteBytesExt};

use crate::types::colors::RED;
use crate::types::player::Player;

mod deop;
mod gamemode;
mod give;
mod op;
mod tps;
mod version;

pub type Handler = for<'a> fn(
    &'a [&'a str],
    &'a mut Player,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;

// the mc client will use this for autocomplete and stuff
// todo: finish
#[derive(Clone, Copy)]
pub enum Parser {
    Integer { min: Option<i32>, max: Option<i32> },
    GameProfile,
    ItemStack,
    Gamemode,
}

impl Parser {
    pub fn id(&self) -> i32 {
        match self {
            Parser::Integer { .. } => 3,
            Parser::GameProfile => 7,
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
    pub requires_op: bool,
    pub handler: Handler,
}

pub static COMMANDS: &[Command] = &[
    Command {
        name: "deop",
        args: &[Arg {
            name: "player",
            parser: Parser::GameProfile,
        }],
        requires_op: true,
        handler: deop::run,
    },
    Command {
        name: "gamemode",
        args: &[Arg {
            name: "mode",
            parser: Parser::Gamemode,
        }],
        requires_op: true,
        handler: gamemode::run,
    },
    Command {
        name: "give",
        args: &[
            Arg {
                name: "item",
                parser: Parser::ItemStack,
            },
            Arg {
                name: "amount",
                parser: Parser::Integer {
                    min: Some(1),
                    max: None,
                },
            },
        ],
        requires_op: true,
        handler: give::run,
    },
    Command {
        name: "op",
        args: &[Arg {
            name: "player",
            parser: Parser::GameProfile,
        }],
        requires_op: true,
        handler: op::run,
    },
    Command {
        name: "tps",
        args: &[],
        requires_op: false,
        handler: tps::run,
    },
    Command {
        name: "version",
        args: &[],
        requires_op: false,
        handler: version::run,
    },
];

pub async fn handle_command(command: String, player: &mut Player) -> anyhow::Result<()> {
    let command_parts = command.split_whitespace().collect::<Vec<&str>>();

    for cmd in COMMANDS {
        if cmd.name == command_parts[0] {
            if cmd.requires_op && !player.is_op {
                player
                    .send_system_message(format!(
                        "{}You do not have permission to use this command.",
                        RED
                    ))
                    .await?;

                return Ok(());
            }

            return (cmd.handler)(&command_parts[1..], player).await;
        }
    }

    player
        .send_system_message(format!("{}Unknown command: {}", RED, command_parts[0]))
        .await?;

    Ok(())
}
