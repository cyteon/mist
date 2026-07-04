pub use blocks::*;

pub struct Property {
    pub name: &'static str,
    pub values: &'static [&'static str],
}

pub struct Block {
    pub name: &'static str,
    pub min_state_id: u16,
    pub default_state: u16,
    pub properties: &'static [Property],
}

#[allow(unused)]
mod blocks {
    use super::{Block, Property};

    include!(concat!(env!("OUT_DIR"), "/blocks.rs"));
}

pub fn block_state_count(block: &Block) -> u32 {
    block
        .properties
        .iter()
        .map(|p| p.values.len() as u32)
        .product::<u32>()
        .max(1)
}

pub fn block_by_state_id(state_id: u16) -> Option<&'static Block> {
    for block in BLOCKS.iter() {
        if state_id >= block.min_state_id
            && state_id < block.min_state_id + block_state_count(block) as u16
        {
            return Some(block);
        }
    }

    None
}

pub fn resolve_state(block: &Block, overrides: Vec<(&'static str, &'static str)>) -> u16 {
    let mut default_remaining = (block.default_state - block.min_state_id) as u32;
    let mut default_indices = vec![0; block.properties.len()];

    for (i, prop) in block.properties.iter().enumerate().rev() {
        let n = prop.values.len() as u32;
        default_indices[i] = default_remaining % n;
        default_remaining /= n;
    }

    let mut offset = 0;
    let mut multiplier = 1;

    for (i, prop) in block.properties.iter().enumerate().rev() {
        let n = prop.values.len() as u32;

        let idx = overrides
            .iter()
            .find(|(name, _)| *name == prop.name)
            .and_then(|(_, value)| prop.values.iter().position(|&v| v == *value))
            .unwrap_or(default_indices[i] as usize) as u32;

        offset += idx * multiplier;
        multiplier *= n;
    }

    block.min_state_id + offset as u16
}

pub fn face_to_axis(face: u8) -> &'static str {
    match face {
        0 | 1 => "y",
        2 | 3 => "z",
        4 | 5 => "x",
        _ => "y",
    }
}

pub fn face_to_direction(face: u8) -> &'static str {
    match face {
        0 => "down",
        1 => "up",
        2 => "north",
        3 => "south",
        4 => "west",
        5 => "east",
        _ => "north",
    }
}

pub fn yaw_to_direction(yaw: f32) -> &'static str {
    let yaw = yaw.rem_euclid(360.0);

    match (((yaw + 45.0) / 90.0) as i32) & 3 {
        0 => "south",
        1 => "west",
        2 => "north",
        _ => "east",
    }
}

pub fn yaw_to_facing(yaw: f32) -> &'static str {
    match yaw_to_direction(yaw) {
        "north" => "south",
        "south" => "north",
        "west" => "east",
        _ => "west",
    }
}

pub fn compute_overrides(block: &Block, face: u8, yaw: f32) -> Vec<(&'static str, &'static str)> {
    let mut overrides = Vec::new();

    if let Some(_) = block.properties.iter().find(|p| p.name == "axis") {
        overrides.push(("axis", face_to_axis(face)));
    }

    if let Some(_) = block.properties.iter().find(|p| p.name == "facing") {
        overrides.push(("facing", yaw_to_facing(yaw)));
    }

    overrides
}

pub fn deepslate_variant(block: u16) -> u16 {
    match block {
        STONE => DEEPSLATE,
        COAL_ORE => DEEPSLATE_COAL_ORE,
        IRON_ORE => DEEPSLATE_IRON_ORE,
        COPPER_ORE => DEEPSLATE_COPPER_ORE,
        GOLD_ORE => DEEPSLATE_GOLD_ORE,
        REDSTONE_ORE => DEEPSLATE_REDSTONE_ORE,
        EMERALD_ORE => DEEPSLATE_EMERALD_ORE,
        LAPIS_ORE => DEEPSLATE_LAPIS_ORE,
        DIAMOND_ORE => DEEPSLATE_DIAMOND_ORE,

        _ => block,
    }
}
