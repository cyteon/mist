use serde::{Deserialize, Serialize};
use anyhow::Context;

#[derive(Serialize, Deserialize)]
pub struct Region {
    pub x: i32,
    pub z: i32,
    pub chunks: Vec<Chunk>,
}

impl Region {
    pub fn to_chunk(&self) -> (i32, i32) {
        (self.x << 5, self.z << 5)
    }

    pub fn new(x: i32, z: i32) -> Self {
        Region {
            x,
            z,
            chunks: Vec::new(),
        }
    }

    pub async fn save(&self) -> anyhow::Result<()> {
        let region_path = format!(
            "{}/regions/{}_{}.mist_region",
            crate::config::SERVER_CONFIG.world_name.clone(),
            self.x,
            self.z
        );

        let serialized = postcard::to_allocvec(self)
            .context("Failed to serialize region")?;
    
        std::fs::write(region_path, serialized)
            .context("Failed to write region to disk")?;
    
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Chunk {
    pub x: i32,
    pub z: i32,
    pub sections: Vec<Section>,
}

impl Chunk {
    pub fn to_region(&self) -> (i32, i32) {
        (self.x >> 5, self.z >> 5)
    }

    pub fn generate(x: i32, z: i32) -> Self {
        // we will use this when proper generation
        let seed = crate::config::SERVER_CONFIG.world_seed as u64;

        // TODO: actual generation

        let mut chunk = Chunk {
            x,
            z,

            sections: (0..24).map(|y| Section::new(y)).collect(),
        };

        for x in 0..16 {
            for z in 0..16 {
                chunk.sections[0].set_block(x, 0, z, 31);
            }
        }

        chunk
    }
}

// 16x16x16 chunk section, 24 per chunk
#[derive(Serialize, Deserialize)]
pub struct Section {
    pub y: i32,
    pub blocks: BlockStorage,
}

impl Section {
    pub fn new(y: i32) -> Self {
        Section {
            y,
            blocks: BlockStorage::new(),
        }
    }

    pub fn set_block(&mut self, x: u8, y: u8, z: u8, block_id: u16) {
        let idx = (y * 16 * 16 + z * 16 + x ) as usize;
        
        let mut palette_index = self.blocks.palette.iter().position(|&id| id == block_id);
        
        if palette_index.is_none() {
            self.blocks.palette.push(block_id);
            palette_index = Some(self.blocks.palette.len() - 1);
            
            let new_bits_per_block = Self::calculate_bits_per_block(self.blocks.palette.len());
            
            if new_bits_per_block > self.blocks.bits_per_block {
                self.blocks.resize_and_repack(new_bits_per_block);
            }
        }
        
        self.blocks.set_palette_index(idx, palette_index.unwrap() as u16);
    }
    
    fn calculate_bits_per_block(palette_size: usize) -> u8 {
        if palette_size == 1 {
            return 0;
        }
        
        let min_bits = (palette_size as f32).log2().ceil() as u8;
        
        match min_bits {
            0..=3 => 4,
            4..=8 => min_bits,
            _ => 15, 
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct BlockStorage {
    pub palette: Vec<u16>,
    pub bits_per_block: u8,
    pub data: Vec<u64>,
}

impl BlockStorage {
    pub fn new() -> Self {
        BlockStorage {
            palette: vec![0],
            bits_per_block: 0,
            data: vec![0u64; 64],
        }
    }

    pub fn resize_and_repack(&mut self, new_bits_per_block: u8) {
        let old_bits = self.bits_per_block as usize;
        let new_bits = new_bits_per_block as usize;
        
        // 16x16x16 = 4096 blocks per section
        let mut indices = Vec::with_capacity(4096);
        for i in 0..4096 {
            indices.push(self.get_palette_index(i, old_bits));
        }
        
        let total_bits = 4096 * new_bits;
        let new_size = (total_bits + 63) / 64;
        
        self.data = vec![0u64; new_size];
        self.bits_per_block = new_bits_per_block;
        
        for (i, &palette_idx) in indices.iter().enumerate() {
            self.set_palette_index(i, palette_idx);
        }
    }
    
    fn get_palette_index(&self, idx: usize, bits: usize) -> u16 {
        let bit_idx = idx * bits;
        let data_idx = bit_idx / 64;
        let bit_offset = bit_idx % 64;
        let mask = (1u64 << bits) - 1;
        
        let mut value = (self.data[data_idx] >> bit_offset) & mask;
        
        if bit_offset + bits > 64 {
            let extra_bits = bit_offset + bits - 64;
            value |= (self.data[data_idx + 1] & ((1u64 << extra_bits) - 1)) << (bits - extra_bits);
        }
        
        value as u16
    }
    
    pub fn set_palette_index(&mut self, idx: usize, palette_index: u16) {
        let bits = self.bits_per_block as usize;
        let bit_idx = idx * bits;
        let data_idx = bit_idx / 64;
        let bit_offset = bit_idx % 64;
        let mask = (1u64 << bits) - 1;
        
        self.data[data_idx] &= !(mask << bit_offset);
        self.data[data_idx] |= ((palette_index as u64) & mask) << bit_offset;
        
        if bit_offset + bits > 64 {
            let extra_bits = bit_offset + bits - 64;
            let extra_mask = (1u64 << extra_bits) - 1;
            self.data[data_idx + 1] &= !extra_mask;
            self.data[data_idx + 1] |= (palette_index as u64 >> (bits - extra_bits)) & extra_mask;
        }
    }
}