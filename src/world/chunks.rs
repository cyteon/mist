use serde::{Deserialize, Serialize};
use anyhow::Context;
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;
use tokio::io::AsyncWriteExt;

use crate::net::codec::write_var;

#[derive(Serialize, Deserialize, Clone)]
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
    
        let compressed = tokio::task::spawn_blocking(move || {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            std::io::copy(&mut &serialized[..], &mut encoder).unwrap();
            encoder.finish().unwrap()
        }).await?;
    
        tokio::fs::write(region_path, compressed).await
            .context("Failed to write region file")?;
    
        Ok(())
    }

    pub async fn load(x: i32, z: i32) -> anyhow::Result<Self> {
        let region_path = format!(
            "{}/regions/{}_{}.mist_region",
            crate::config::SERVER_CONFIG.world_name.clone(),
            x,
            z
        );

        let compressed = tokio::fs::read(region_path).await
            .context("Failed to read region file")?;
    
        let serialized = tokio::task::spawn_blocking(move || {
            let mut decoder = ZlibDecoder::new(&compressed[..]);
            let mut decompressed = Vec::new();
            std::io::copy(&mut decoder, &mut decompressed).unwrap();
            decompressed
        }).await?;
    
        let region: Region = postcard::from_bytes(&serialized)
            .context("Failed to deserialize region")?;
    
        Ok(region)
    }
}

#[derive(Serialize, Deserialize, Clone)]
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
                chunk.sections[0].set_block(
                    x, 0, z, 
                    crate::types::blocks::BLOCKS.get("minecraft:grass_block").unwrap().default
                ); // should be grass
            }
        }

        chunk
    }
}

#[derive(Serialize, Deserialize, Clone)]
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
        let idx = (y as usize * 16 * 16) + (z as usize * 16) + (x as usize);
        
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
        let bits = min_bits.max(4);
        
        match bits {
            0 => 0,
            4..=8 => bits,
            _ => 15, 
        }
    }

    pub fn block_count(&self) -> i16 {
        let mut count = 0i16;
        
        for i in 0..4096 {
            let palette_idx = self.blocks.get_palette_index(i, self.blocks.bits_per_block as usize);
            if let Some(&block_id) = self.blocks.palette.get(palette_idx as usize) {
                if block_id != 0 {
                    count += 1;
                }
            }
        }
        count
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BlockStorage {
    pub palette: Vec<u16>,
    pub bits_per_block: u8,
    pub data: Vec<i64>,
}

impl BlockStorage {
    pub fn new() -> Self {
        BlockStorage {
            palette: vec![0],
            bits_per_block: 0,
            data: Vec::new(),
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
        
        self.data = vec![0i64; new_size];
        self.bits_per_block = new_bits_per_block;
        
        for (i, &palette_idx) in indices.iter().enumerate() {
            self.set_palette_index(i, palette_idx);
        }
    }
    
    fn get_palette_index(&self, idx: usize, bits: usize) -> u16 {
        if bits == 0 {
            return 0;
        }

        let bit_idx = idx * bits;
        let data_idx = bit_idx / 64;
        let bit_offset = bit_idx % 64;
        let mask = (1i64 << bits) - 1;
        
        let mut value = (self.data[data_idx] >> bit_offset) & mask;
        
        if bit_offset + bits > 64 {
            let extra_bits = bit_offset + bits - 64;
            value |= (self.data[data_idx + 1] & ((1i64 << extra_bits) - 1)) << (bits - extra_bits);
        }
        
        value as u16
    }
    
    pub fn set_palette_index(&mut self, idx: usize, palette_index: u16) {
        if self.bits_per_block == 0 {
            return;
        }

        let bits = self.bits_per_block as usize;
        let bit_idx = idx * bits;
        let data_idx = bit_idx / 64;
        let bit_offset = bit_idx % 64;
        let mask = (1i64 << bits) - 1;
        
        self.data[data_idx] &= !(mask << bit_offset);
        self.data[data_idx] |= ((palette_index as i64) & mask) << bit_offset;
        
        if bit_offset + bits > 64 {
            let extra_bits = bit_offset + bits - 64;
            let extra_mask = (1i64 << extra_bits) - 1;
            self.data[data_idx + 1] &= !extra_mask;
            self.data[data_idx + 1] |= (palette_index as i64 >> (bits - extra_bits)) & extra_mask;
        }
    }

    pub async fn write_paletted_container<W: AsyncWriteExt + Unpin>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u8(self.bits_per_block).await?;
        
        match self.bits_per_block {
            0 => {
                write_var(writer, self.palette[0] as i32).await?;
            }

            4..=8 => {
                write_var(writer, self.palette.len() as i32).await?;
                for &block_id in &self.palette {
                    write_var(writer, block_id as i32).await?;
                }
                
                //write_var(writer, self.data.len() as i32).await?;
                for &value in &self.data {
                    writer.write_i64(value).await?;
                }
            }

            15 => {
                todo!("Direct palette storage not implemented");
            }
            
            _ => {
                anyhow::bail!("Invalid bits_per_block value: {}", self.bits_per_block);
            }
        }
        
        Ok(())
    }
}