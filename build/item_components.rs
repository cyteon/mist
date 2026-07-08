use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

pub fn encode_item_components() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();

    let item_data_path = Path::new(&manifest).join("src/assets/item_data.json");
    let item_protocol_path = Path::new(&manifest).join("src/assets/items.json");

    println!("cargo:rerun-if-changed={}", item_data_path.display());
    println!("cargo:rerun-if-changed={}", item_protocol_path.display());

    let item_data_bytes = fs::read(&item_data_path).expect("Failed to read item_data.json");
    let item_protocol_bytes = fs::read(&item_protocol_path).expect("Failed to read items.json");

    let item_data: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&item_data_bytes).expect("Failed to parse item_data.json");
    let item_protocol: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&item_protocol_bytes).expect("Failed to parse items.json");

    let mut out = String::new();
    out.push_str("pub fn get_item_components(item_id: i32) -> Vec<(i32, Vec<u8>)> {\n");
    out.push_str("    match item_id {\n");

    for (name, data) in item_data {
        let protocol_id = item_protocol["entries"][&name]["protocol_id"]
            .as_i64()
            .unwrap() as i32;

        let mut entries = Vec::new();

        if let Some(tool) = data["components"].get("minecraft:tool") {
            entries.push((28, encode_tool_component(tool)));
        }

        if !entries.is_empty() {
            out.push_str(&format!("        {} => vec![\n", protocol_id));

            for (comp_id, comp_data) in entries {
                out.push_str(&format!(
                    "            ({}, vec!{:?}),\n",
                    comp_id, comp_data
                ));
            }

            out.push_str("        ],\n");
        }
    }

    out.push_str("        _ => vec![],\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("item_components.rs");
    fs::write(out_path, out).unwrap();
}

const KNOWN_TAGS: &[&str] = &[
    "#minecraft:mineable/pickaxe",
    "#minecraft:mineable/shovel",
    "#minecraft:mineable/axe",
    "#minecraft:mineable/hoe",
];

fn encode_tool_component(tool: &serde_json::Value) -> Vec<u8> {
    let mut buffer = Vec::new();
    let rules = tool["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|rule| {
            let blocks = rule["blocks"].as_str().unwrap_or("");

            if blocks.starts_with('#') {
                KNOWN_TAGS.contains(&blocks)
            } else {
                true
            }
        })
        .collect::<Vec<_>>();

    write_var(&mut buffer, rules.len() as i32);

    for rule in rules {
        if let Some(blocks) = rule["blocks"].as_str() {
            write_var(&mut buffer, 0);
            write_string(&mut buffer, &blocks[1..]);
        } else {
            write_var(&mut buffer, 2);
            write_var(&mut buffer, 0);
        }

        if let Some(speed) = rule.get("speed").and_then(|s| s.as_f64()) {
            buffer.push(true as u8);
            buffer.extend_from_slice(&(speed as f32).to_be_bytes());
        } else {
            buffer.push(false as u8);
        }

        if let Some(correct) = rule.get("correct_for_drops").and_then(|c| c.as_bool()) {
            buffer.push(true as u8);
            buffer.push(correct as u8);
        } else {
            buffer.push(false as u8);
        }
    }

    buffer.extend_from_slice(&1.0f32.to_be_bytes());

    if let Some(damage_per_block) = tool.get("damage_per_block").and_then(|d| d.as_i64()) {
        write_var(&mut buffer, damage_per_block as i32);
    } else {
        write_var(&mut buffer, 1);
    }

    if let Some(can_destroy_blocks_in_creative) = tool
        .get("can_destroy_blocks_in_creative")
        .and_then(|c| c.as_bool())
    {
        buffer.push(can_destroy_blocks_in_creative as u8);
    } else {
        buffer.push(true as u8);
    }

    buffer
}

pub fn write_var(buffer: &mut Vec<u8>, value: i32) {
    let mut value = value as u32;

    loop {
        let mut temp = (value & 0b01111111) as u8;

        value >>= 7;

        if value != 0 {
            temp |= 0b10000000;
        }

        buffer.push(temp);

        if value == 0 {
            break;
        }
    }
}

pub fn write_string(buffer: &mut Vec<u8>, value: &str) {
    write_var(buffer, value.len() as i32);
    buffer.extend_from_slice(value.as_bytes());
}
