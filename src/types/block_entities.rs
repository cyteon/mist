use std::collections::HashMap;

use crate::types::items::ItemStack;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum BlockEntityData {
    Chest { inventory: [Option<ItemStack>; 27] },
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
}
