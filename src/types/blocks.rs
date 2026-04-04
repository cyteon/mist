pub use blocks::*;

#[allow(unused)]
mod blocks {
    include!(concat!(env!("OUT_DIR"), "/blocks.rs"));
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
