pub use items::*;

#[allow(unused)]
mod items {
    include!(concat!(env!("OUT_DIR"), "/items.rs"));
}

include!(concat!(env!("OUT_DIR"), "/item_to_block.rs"));