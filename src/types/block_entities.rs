use std::collections::HashMap;

use crate::types::{
    blocks::{self, block_by_state_id},
    items::ItemStack,
};

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub enum BlockEntityData {
    Chest {
        items: [Option<ItemStack>; 27],
        #[serde(skip)]
        viewers: Vec<String>,
    },
}

impl BlockEntityData {
    pub fn write_nbt(&self, w: &mut Vec<u8>) -> anyhow::Result<()> {
        match self {
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
            BlockEntityData::Chest { .. } => 1,
        }
    }
}

pub fn get_block_entity(block_id: u16) -> Option<BlockEntityData> {
    match block_by_state_id(block_id).map(|b| b.default_state) {
        Some(blocks::CHEST) => Some(BlockEntityData::Chest {
            items: [None; 27],
            viewers: Vec::new(),
        }),

        _ => None,
    }
}
