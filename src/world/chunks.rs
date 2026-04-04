use anyhow::Context;
use byteorder::{BigEndian, WriteBytesExt};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use serde::{Deserialize, Serialize};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::net::codec::write_var;

#[derive(Serialize, Deserialize, Clone)]
pub struct Region {
    pub x: i32,
    pub z: i32,
    pub chunks: Vec<Chunk>,
}

impl Region {
    pub fn new(x: i32, z: i32) -> Self {
        Region {
            x,
            z,
            chunks: Vec::new(),
        }
    }

    pub fn get_chunk(&self, x: i32, z: i32) -> Option<&Chunk> {
        self.chunks
            .iter()
            .find(|chunk| chunk.x == x && chunk.z == z)
    }

    pub async fn save(&self) -> anyhow::Result<()> {
        let region_path = format!(
            "{}/regions/{}_{}.mist_region",
            crate::config::SERVER_CONFIG.world_name.clone(),
            self.x,
            self.z
        );

        let serialized = postcard::to_allocvec(self).context("Failed to serialize region")?;

        let compressed = tokio::task::spawn_blocking(move || {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            std::io::copy(&mut &serialized[..], &mut encoder).unwrap();
            encoder.finish().unwrap()
        })
        .await?;

        tokio::fs::write(region_path, compressed)
            .await
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

        let compressed = tokio::fs::read(region_path)
            .await
            .context("Failed to read region file")?;

        let serialized = tokio::task::spawn_blocking(move || {
            let mut decoder = ZlibDecoder::new(&compressed[..]);
            let mut decompressed = Vec::new();
            std::io::copy(&mut decoder, &mut decompressed).unwrap();
            decompressed
        })
        .await?;

        let region: Region =
            postcard::from_bytes(&serialized).context("Failed to deserialize region")?;

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
    pub fn set_block(&mut self, x: u8, y: i32, z: u8, block_id: u16) {
        let section_idx = y.div_euclid(16) + 4;

        if let Some(section) = self.sections.get_mut(section_idx as usize) {
            section.set_block(x, (y & 15) as u8, z, block_id);
        }
    }

    pub fn get_block(&self, x: u8, y: i32, z: u8) -> u16 {
        let section_idx = y.div_euclid(16) + 4;

        if let Some(section) = self.sections.get(section_idx as usize) {
            let idx = ((y & 15) as usize * 16 * 16) + (z as usize * 16) + (x as usize);
            let palette_idx = section
                .blocks
                .get_palette_index(idx, section.blocks.bits_per_block as usize);
            return section
                .blocks
                .palette
                .get(palette_idx as usize)
                .copied()
                .unwrap_or(0);
        }

        0
    }

    pub fn get_surface_y(&self, x: u8, z: u8) -> i32 {
        for section in self.sections.iter().rev() {
            for y in (0..16).rev() {
                let idx = (y as usize * 16 * 16) + (z as usize * 16) + (x as usize);
                let palette_idx = section
                    .blocks
                    .get_palette_index(idx, section.blocks.bits_per_block as usize);
                let block_id = section
                    .blocks
                    .palette
                    .get(palette_idx as usize)
                    .copied()
                    .unwrap_or(0);

                if block_id != 0 {
                    return (section.y * 16) + y - 64;
                }
            }
        }

        -64
    }

    pub fn get_surface_y_below_point(&self, x: u8, y: i32, z: u8) -> i32 {
        let mut current_y = y;

        while current_y >= -64 {
            let block_id = self.get_block(x, current_y, z);

            if block_id != 0 {
                return current_y;
            }

            current_y -= 1;
        }

        -64
    }

    pub fn chunk_seed(&self) -> u64 {
        let seed = crate::config::SERVER_CONFIG.world_seed;
        let mut hasher = DefaultHasher::new();

        seed.hash(&mut hasher);
        self.x.hash(&mut hasher);
        self.z.hash(&mut hasher);

        hasher.finish()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Section {
    pub y: i32,
    pub blocks: BlockStorage,
    pub block_count: i16,
}

impl Section {
    pub fn new(y: i32) -> Self {
        Section {
            y,
            blocks: BlockStorage::new(),
            block_count: 0,
        }
    }

    pub fn set_block(&mut self, x: u8, y: u8, z: u8, block_id: u16) {
        let idx = (y as usize * 16 * 16) + (z as usize * 16) + (x as usize);
        let old_palette_idx = self
            .blocks
            .get_palette_index(idx, self.blocks.bits_per_block as usize);
        let old_block = self
            .blocks
            .palette
            .get(old_palette_idx as usize)
            .copied()
            .unwrap_or(0);
        let mut palette_index = self.blocks.palette.iter().position(|&id| id == block_id);

        if old_block == 0 && block_id != 0 {
            self.block_count += 1;
        } else if old_block != 0 && block_id == 0 {
            self.block_count -= 1;
        }

        if palette_index.is_none() {
            self.blocks.palette.push(block_id);
            palette_index = Some(self.blocks.palette.len() - 1);

            let new_bits_per_block = Self::calculate_bits_per_block(self.blocks.palette.len());

            if new_bits_per_block > self.blocks.bits_per_block {
                self.blocks.resize_and_repack(new_bits_per_block);
            }
        }

        self.blocks
            .set_palette_index(idx, palette_index.unwrap() as u16);
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

    pub fn write_paletted_container<W: WriteBytesExt + Unpin>(
        &self,
        writer: &mut W,
    ) -> anyhow::Result<()> {
        writer.write_u8(self.bits_per_block)?;

        match self.bits_per_block {
            0 => {
                write_var(writer, self.palette[0] as i32)?;
            }

            4..=8 => {
                write_var(writer, self.palette.len() as i32)?;
                for &block_id in &self.palette {
                    write_var(writer, block_id as i32)?;
                }

                for &value in &self.data {
                    writer.write_i64::<BigEndian>(value)?;
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
