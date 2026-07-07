use std::collections::HashMap;

use crate::types::{
    blocks::{self, block_by_state_id},
    items::ItemStack,
    recipes::get_smelting_recipe,
};

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub enum BlockEntityData {
    Chest {
        items: [Option<ItemStack>; 27],
        #[serde(skip)]
        viewers: Vec<String>,
    },

    Furnace {
        input: Option<ItemStack>,
        fuel: Option<ItemStack>,
        output: Option<ItemStack>,
        currently_cooking: Option<ItemStack>,
        lit_left: u32,
        lit_total: u32,
        cook_left: u32,
        #[serde(skip)]
        properties_changed: bool,
        #[serde(skip)]
        slots_changed: bool,
    },
}

impl BlockEntityData {
    pub fn write_nbt(&self, w: &mut Vec<u8>) -> anyhow::Result<()> {
        match self {
            BlockEntityData::Furnace { .. } => {
                let compound = craftflow_nbt::DynNBT::Compound(HashMap::from([(
                    "id".to_string(),
                    craftflow_nbt::DynNBT::String("minecraft:furnace".to_string()),
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

    pub fn tick(&mut self) {
        match self {
            BlockEntityData::Furnace {
                input,
                fuel,
                output,
                currently_cooking,
                lit_left,
                lit_total,
                cook_left,
                properties_changed,
                slots_changed,
            } => {
                *properties_changed = false;
                *slots_changed = false;

                if *lit_left > 0 {
                    *lit_left -= 1;
                    *properties_changed = true;
                }

                if *lit_left == 0
                    && let Some(stack) = fuel
                {
                    stack.count -= 1;

                    if stack.count == 0 {
                        *fuel = None;
                    }

                    *lit_left = 200;
                    *lit_total = 200;

                    *properties_changed = true;
                    *slots_changed = true;
                }

                if *cook_left > 0 && *lit_left > 0 {
                    *cook_left -= 1;
                    *properties_changed = true;
                }

                if *cook_left == 0 {
                    if let Some(stack) = output {
                        stack.count += 1;
                    } else {
                        *output = currently_cooking.take();
                    }

                    *currently_cooking = None;

                    if let Some(input_stack) = input {
                        let result_id = get_smelting_recipe(input_stack.item_id);

                        if let Some(result_id) = result_id {
                            if let Some(output_stack) = output {
                                if output_stack.item_id == result_id && output_stack.count < 64 {
                                    *currently_cooking = Some(ItemStack {
                                        item_id: result_id,
                                        count: 1,
                                    });

                                    *cook_left = 200;
                                    input_stack.count -= 1;
                                }
                            } else {
                                *currently_cooking = Some(ItemStack {
                                    item_id: result_id,
                                    count: 1,
                                });

                                *cook_left = 200;
                                input_stack.count -= 1;
                            }
                        }

                        if input_stack.count == 0 {
                            *input = None;
                        }
                    }

                    *properties_changed = true;
                    *slots_changed = true;
                }
            }

            _ => {}
        }
    }
}

pub fn get_block_entity(block_id: u16) -> Option<BlockEntityData> {
    match block_by_state_id(block_id).map(|b| b.default_state) {
        Some(blocks::FURNACE) => Some(BlockEntityData::Furnace {
            input: None,
            fuel: None,
            output: None,
            currently_cooking: None,
            lit_left: 0,
            lit_total: 0,
            cook_left: 200,
            properties_changed: false,
            slots_changed: false,
        }),

        Some(blocks::CHEST) => Some(BlockEntityData::Chest {
            items: [None; 27],
            viewers: Vec::new(),
        }),

        _ => None,
    }
}
