pub use blocks::*;

#[allow(unused)]
mod blocks {
    include!(concat!(env!("OUT_DIR"), "/blocks.rs"));
}