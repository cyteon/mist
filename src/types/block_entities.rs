use std::collections::HashMap;

use crate::{
    net::packets::clientbound::block_update::send_block_update,
    types::{
        blocks::{self, block_by_state_id, with_ovveride},
        items::{self, ItemStack},
        player::broadcast_packet,
        recipes::{get_blasting_recipe, get_smelting_recipe, get_smoking_recipe},
        tags::TAGS,
    },
    world::chunks::Chunk,
};

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub enum FurnaceType {
    Furnace,
    BlastFurnace,
    Smoker,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub enum BlockEntityData {
    Chest {
        items: [Option<ItemStack>; 27],
        #[serde(skip)]
        viewers: Vec<String>,
    },

    Furnace {
        furnace_type: FurnaceType,
        input: Option<ItemStack>,
        fuel: Option<ItemStack>,
        output: Option<ItemStack>,
        currently_cooking: Option<ItemStack>,
        lit_left: u32,
        lit_total: u32,
        cook_left: u32,
        // seperate for this one cause both cook_left and lit_left change at same rate
        // while lit_total only changes when fuel type is changed
        #[serde(skip)]
        lit_total_last: u32,
        #[serde(skip)]
        properties_changed: bool,
        #[serde(skip)]
        slots_changed: bool,
        #[serde(skip)]
        was_lit: bool,
    },
}

impl BlockEntityData {
    pub fn write_nbt(&self, w: &mut Vec<u8>) -> anyhow::Result<()> {
        match self {
            BlockEntityData::Furnace { furnace_type, .. } => {
                let id = match furnace_type {
                    FurnaceType::Furnace => "minecraft:furnace",
                    FurnaceType::BlastFurnace => "minecraft:blast_furnace",
                    FurnaceType::Smoker => "minecraft:smoker",
                };

                let compound = craftflow_nbt::DynNBT::Compound(HashMap::from([(
                    "id".to_string(),
                    craftflow_nbt::DynNBT::String(id.to_string()),
                )]));

                craftflow_nbt::to_writer(w, &compound)?;
            }

            BlockEntityData::Chest { .. } => {
                let compound = craftflow_nbt::DynNBT::Compound(HashMap::from([(
                    "id".to_string(),
                    craftflow_nbt::DynNBT::String("minecraft:chest".to_string()),
                )]));

                craftflow_nbt::to_writer(w, &compound)?;
            }
        }
        Ok(())
    }

    pub fn type_id(&self) -> i32 {
        match self {
            BlockEntityData::Furnace { .. } => 0,
            BlockEntityData::Chest { .. } => 1,
        }
    }

    pub async fn tick(&mut self, chunk: &mut Chunk, cords: (i32, i32, i32)) -> anyhow::Result<()> {
        match self {
            BlockEntityData::Furnace {
                furnace_type,
                input,
                fuel,
                output,
                currently_cooking,
                lit_left,
                lit_total,
                cook_left,
                lit_total_last,
                properties_changed,
                slots_changed,
                was_lit,
            } => {
                *properties_changed = false;
                *slots_changed = false;
                *lit_total_last = *lit_total;

                if *lit_left > 0 {
                    *lit_left -= 1;
                    *properties_changed = true;
                }

                if *lit_left == 0
                    && let Some(stack) = fuel
                    && (currently_cooking.is_some() || input.is_some())
                {
                    let recipe = input.as_ref().and_then(|stack| match furnace_type {
                        FurnaceType::Furnace => get_smelting_recipe(stack.item_id),
                        FurnaceType::BlastFurnace => get_blasting_recipe(stack.item_id),
                        FurnaceType::Smoker => get_smoking_recipe(stack.item_id),
                    });

                    let can_smelt = match (recipe, output.as_ref()) {
                        (Some(result_id), Some(output_stack)) => {
                            output_stack.item_id == result_id && output_stack.count < 64
                        }

                        (Some(_), None) => true,
                        _ => false,
                    };

                    if (can_smelt || currently_cooking.is_some())
                        && let Some(fuel_time) = fuel_time(stack.item_id)
                    {
                        stack.count -= 1;

                        if stack.count == 0 {
                            *fuel = None;
                        }

                        *lit_left = fuel_time;
                        *lit_total = fuel_time;

                        *properties_changed = true;
                        *slots_changed = true;
                    }
                }

                if *cook_left > 0 && *lit_left > 0 && currently_cooking.is_some() {
                    *cook_left -= 1;
                    *properties_changed = true;
                }

                if currently_cooking.is_none() && *lit_left > 0 {
                    if let Some(input_stack) = input {
                        let result_id = match furnace_type {
                            FurnaceType::Furnace => get_smelting_recipe(input_stack.item_id),
                            FurnaceType::BlastFurnace => get_blasting_recipe(input_stack.item_id),
                            FurnaceType::Smoker => get_smoking_recipe(input_stack.item_id),
                        };

                        if let Some(result_id) = result_id {
                            if let Some(output_stack) = output {
                                if output_stack.item_id == result_id && output_stack.count < 64 {
                                    *currently_cooking = Some(ItemStack {
                                        item_id: result_id,
                                        count: 1,
                                    });

                                    input_stack.count -= 1;

                                    *cook_left = match furnace_type {
                                        FurnaceType::Furnace => 200,
                                        _ => 100,
                                    };

                                    *slots_changed = true;
                                    *properties_changed = true;
                                }
                            } else {
                                *currently_cooking = Some(ItemStack {
                                    item_id: result_id,
                                    count: 1,
                                });

                                input_stack.count -= 1;

                                *cook_left = match furnace_type {
                                    FurnaceType::Furnace => 200,
                                    _ => 100,
                                };

                                *slots_changed = true;
                                *properties_changed = true;
                            }
                        }

                        if input_stack.count == 0 {
                            *input = None;
                        }
                    }
                }

                if *cook_left == 0 {
                    if currently_cooking.is_some() {
                        if let Some(stack) = output {
                            stack.count += 1;
                        } else {
                            *output = currently_cooking.take();
                        }

                        *cook_left = match furnace_type {
                            FurnaceType::Furnace => 200,
                            _ => 100,
                        };

                        *currently_cooking = None;
                        *properties_changed = true;
                        *slots_changed = true;
                    }
                }

                if (*lit_left > 0 && !*was_lit) || (*lit_left == 0 && *was_lit) {
                    let block = chunk.get_block(cords.0 as u8, cords.1, cords.2 as u8);

                    let new =
                        with_ovveride(block, "lit", if *lit_left > 0 { "true" } else { "false" });

                    chunk.set_block(cords.0 as u8, cords.1, cords.2 as u8, new);

                    let wx = chunk.x * 16 + cords.0;
                    let wz = chunk.z * 16 + cords.2;

                    let mut buffer = Vec::new();
                    send_block_update(&mut buffer, wx, cords.1, wz, new as i32).await?;
                    broadcast_packet(buffer, (wx as f64, cords.1 as f64, wz as f64), None).await?;
                }

                *was_lit = *lit_left > 0;
            }

            _ => {}
        }

        Ok(())
    }
}

pub fn get_block_entity(block_id: u16) -> Option<BlockEntityData> {
    match block_by_state_id(block_id).map(|b| b.default_state) {
        Some(blocks::FURNACE) => Some(BlockEntityData::Furnace {
            furnace_type: FurnaceType::Furnace,
            input: None,
            fuel: None,
            output: None,
            currently_cooking: None,
            lit_left: 0,
            lit_total: 0,
            cook_left: 200,
            lit_total_last: 0,
            properties_changed: false,
            slots_changed: false,
            was_lit: false,
        }),

        Some(blocks::BLAST_FURNACE) => Some(BlockEntityData::Furnace {
            furnace_type: FurnaceType::BlastFurnace,
            input: None,
            fuel: None,
            output: None,
            currently_cooking: None,
            lit_left: 0,
            lit_total: 0,
            cook_left: 100,
            lit_total_last: 0,
            properties_changed: false,
            slots_changed: false,
            was_lit: false,
        }),

        Some(blocks::SMOKER) => Some(BlockEntityData::Furnace {
            furnace_type: FurnaceType::Smoker,
            input: None,
            fuel: None,
            output: None,
            currently_cooking: None,
            lit_left: 0,
            lit_total: 0,
            cook_left: 100,
            lit_total_last: 0,
            properties_changed: false,
            slots_changed: false,
            was_lit: false,
        }),

        Some(blocks::CHEST) => Some(BlockEntityData::Chest {
            items: [None; 27],
            viewers: Vec::new(),
        }),

        _ => None,
    }
}

// todo: finish
pub fn fuel_time(item_id: i32) -> Option<u32> {
    match item_id {
        items::LAVA_BUCKET => return Some(20000),
        items::COAL_BLOCK => return Some(16000),
        items::DRIED_KELP_BLOCK => return Some(4000),
        items::BLAZE_ROD => return Some(2400),
        items::COAL | items::CHARCOAL => return Some(1600),
        _ => {}
    }

    let types: &[(&str, u32)] = &[
        ("minecraft:boats", 1200),
        ("minecraft:hanging_signs", 800),
        ("minecraft:planks", 300),
        ("minecraft:logs_that_burn", 300),
        ("minecraft:fence_gates", 300),
        ("minecraft:wooden_stairs", 300),
        ("minecraft:wooden_fences", 300),
        ("minecraft:wooden_pressure_plates", 300),
        ("minecraft:wooden_trapdoors", 300),
        ("minecraft:signs", 200),
        ("minecraft:wooden_doors", 200),
        ("minecraft:wooden_slabs", 150),
        ("minecraft:saplings", 100),
        ("minecraft:wooden_buttons", 100),
        ("minecraft:wool", 100),
        ("minecraft:wool_carpets", 67),
    ];

    let item_tags = TAGS
        .iter()
        .find(|(reg, _)| *reg == "minecraft:item")
        .map(|(_, tags)| *tags)
        .unwrap_or(&[]);

    for (tag, burn_time) in types {
        if let Some(found) = item_tags.iter().find(|(name, _)| name == tag) {
            if found.1.contains(&item_id) {
                return Some(*burn_time);
            }
        }
    }

    None
}
