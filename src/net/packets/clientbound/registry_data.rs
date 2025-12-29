use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::BTreeMap;
use tokio::io::AsyncWriteExt;
use crate::net::codec::write_var;

#[derive(Debug, Clone)]
pub struct RegistryEntry {
    pub id: String,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct RegistryDataPacket {
    pub registry_id: String,
    pub entries: Vec<RegistryEntry>,
}

pub static REGISTRY_PACKETS: Lazy<Vec<RegistryDataPacket>> = Lazy::new(|| {
    process_registry_packets()
});

fn process_registry_packets() -> Vec<RegistryDataPacket> {
    let json_file = include_bytes!("../../../assets/registries.json");
    let registry_map: BTreeMap<String, BTreeMap<String, Value>> =
        serde_json::from_slice(json_file).expect("Failed to parse registries.json");

    registry_map
        .into_iter()
        .map(|(registry_id, entries)| {
            let mut packets: Vec<RegistryEntry> = Vec::with_capacity(entries.len());
            for (entry_id, value) in entries {
                let mut nbt_buf = Vec::new();
                let nbt_value = match value {
                    Value::Object(_) => value,
                    other => {
                        let mut obj = serde_json::Map::new();
                        obj.insert("value".to_string(), other);
                        Value::Object(obj)
                    }
                };
                craftflow_nbt::to_writer(&mut nbt_buf, &nbt_value).expect("Failed to write NBT");
                let packet_entry = RegistryEntry {
                    id: entry_id,
                    data: if nbt_buf.is_empty() { None } else { Some(nbt_buf) },
                };
                packets.push(packet_entry);
            }
            RegistryDataPacket {
                registry_id,
                entries: packets,
            }
        })
        .collect()
}

pub async fn send_all_registers<W: AsyncWriteExt + Unpin>(
    stream: &mut W,
) -> anyhow::Result<()> {
    for packet in REGISTRY_PACKETS.iter() {
        let mut packet_data = vec![0x07];
        write_var(&mut packet_data, packet.registry_id.len() as i32).await?;
        packet_data.extend_from_slice(packet.registry_id.as_bytes());
        write_var(&mut packet_data, packet.entries.len() as i32).await?;
        for entry in packet.entries.iter() {
            write_var(&mut packet_data, entry.id.len() as i32).await?;
            packet_data.extend_from_slice(entry.id.as_bytes());
            match &entry.data {
                None => packet_data.push(0),
                Some(nbt_bytes) => {
                    packet_data.push(1);
                    packet_data.extend_from_slice(nbt_bytes);
                }
            }
        }
        write_var(stream, packet_data.len() as i32).await?;
        stream.write_all(&packet_data).await?;
        stream.flush().await?;
    }
    Ok(())
}
