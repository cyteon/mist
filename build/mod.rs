mod blocks;
mod food_data;
mod item_components;
mod items;
mod packets;
mod recipes;
mod tags;

fn main() {
    packets::load_packets();
    blocks::load_blocks();
    items::load_items();
    items::load_item_to_block();
    recipes::load_recipes();
    item_components::encode_item_components();
    tags::load_tags();
    food_data::load_food_data();
    recipes::load_furnace_recipes();
}
