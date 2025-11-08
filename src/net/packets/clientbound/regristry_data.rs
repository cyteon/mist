use std::collections::HashMap;

use fastnbt::Value;
use serde_json::json;

use crate::net::codec::write_var;

pub async fn send_regristry_data<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![];
    write_var(&mut packet_data, 0x07).await?;
    
    // only sending one critical registry

    let registry_id = "minecraft:dimension_type";
    write_var(&mut packet_data, registry_id.len() as i32).await?;
    packet_data.extend_from_slice(registry_id.as_bytes());

    write_var(&mut packet_data, 1).await?;

    let entry_id = "minecraft:overworld";
    write_var(&mut packet_data, entry_id.len() as i32).await?;
    packet_data.extend_from_slice(entry_id.as_bytes());

    packet_data.push(0u8);

    let mut overworld = HashMap::new();
    overworld.insert("has_skylight".to_string(), Value::Byte(1));
    overworld.insert("has_ceiling".to_string(), Value::Byte(0));
    overworld.insert("ultrawarm".to_string(), Value::Byte(0));
    overworld.insert("natural".to_string(), Value::Byte(1));
    overworld.insert("coordinate_scale".to_string(), Value::Double(1.0)); // DOUBLE
    overworld.insert("bed_works".to_string(), Value::Byte(1));
    overworld.insert("respawn_anchor_works".to_string(), Value::Byte(1));
    overworld.insert("min_y".to_string(), Value::Int(-64));
    overworld.insert("height".to_string(), Value::Int(384));
    overworld.insert("logical_height".to_string(), Value::Int(384));
    overworld.insert("infiniburn".to_string(), Value::String("#minecraft:infiniburn_overworld".to_string()));
    overworld.insert("effects".to_string(), Value::String("minecraft:overworld".to_string()));
    overworld.insert("ambient_light".to_string(), Value::Double(0.0)); // DOUBLE
    overworld.insert("piglin_safe".to_string(), Value::Byte(0));
    overworld.insert("has_raids".to_string(), Value::Byte(1));
    overworld.insert("monster_spawn_light_level".to_string(), Value::Compound({
        let mut light = HashMap::new();
        light.insert("min_inclusive".to_string(), Value::Int(0));
        light.insert("max_inclusive".to_string(), Value::Int(7));
        light.insert("type".to_string(), Value::String("minecraft:uniform".to_string()));
        light
    }));
    overworld.insert("monster_spawn_block_light_limit".to_string(), Value::Byte(0));

    let nbt_value = Value::Compound(overworld);
    let nbt_bytes = fastnbt::to_bytes_with_opts(&nbt_value, fastnbt::SerOpts::network_nbt())?;

    write_var(&mut packet_data, nbt_bytes.len() as i32).await?;
    packet_data.extend_from_slice(&nbt_bytes);

    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}