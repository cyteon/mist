pub use items::*;

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct ItemStack {
    pub item_id: i32,
    pub count: u8,
}

#[allow(unused)]
mod items {
    include!(concat!(env!("OUT_DIR"), "/items.rs"));
}

include!(concat!(env!("OUT_DIR"), "/item_to_block.rs"));
