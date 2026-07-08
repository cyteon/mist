use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

pub fn load_items() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let json_path = Path::new(&manifest).join("src/assets/items.json");
    println!("cargo:rerun-if-changed={}", json_path.display());

    let bytes = fs::read(&json_path).expect("Failed to read items.json");

    let json: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&bytes).expect("Failed to parse items.json");

    let mut out = String::new();

    for (key, item) in json["entries"].as_object().unwrap() {
        out.push_str(&format!(
            "pub const {}: i32 = {};\n",
            key.to_uppercase()
                .replace("MINECRAFT:", "")
                .replace("/", "_"),
            item["protocol_id"].as_u64().unwrap()
        ));
    }

    out.push_str("pub fn get_item_id(name: &str) -> i32 {\n");
    out.push_str("    match name {\n");

    for (key, item) in json["entries"].as_object().unwrap() {
        out.push_str(&format!(
            "        \"{}\" => {},\n",
            key,
            item["protocol_id"].as_u64().unwrap()
        ));
    }

    out.push_str("        _ => 0,\n"); // default is air
    out.push_str("    }\n");
    out.push_str("}\n");

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("items.rs");
    fs::write(out_path, out).unwrap();
}

pub fn load_item_to_block() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let blocks_json_path = Path::new(&manifest).join("src/assets/blocks.json");
    let items_json_path = Path::new(&manifest).join("src/assets/items.json");
    println!("cargo:rerun-if-changed={}", blocks_json_path.display());

    let block_bytes = fs::read(&blocks_json_path).expect("Failed to read blocks.json");
    let item_bytes = fs::read(&items_json_path).expect("Failed to read items.json");

    let blocks: serde_json::Value =
        serde_json::from_slice(&block_bytes).expect("Failed to parse blocks.json");
    let items: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&item_bytes).expect("Failed to parse items.json");

    let mut out = String::new();
    out.push_str("pub fn item_to_block(item_id: i32) -> Option<u16> {\n");
    out.push_str("    match item_id {\n");

    for (ik, iv) in items["entries"].as_object().unwrap() {
        let item_id = iv["protocol_id"].as_i64().unwrap() as i32;
        let block_name = ik.replace("minecraft:", "");
        let block_id = blocks
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["name"].as_str().unwrap() == block_name)
            .and_then(|b| b["defaultState"].as_i64())
            .unwrap_or(-1) as u16;

        if block_id != u16::MAX {
            out.push_str(&format!("        {} => Some({}),\n", item_id, block_id));
        }
    }

    out.push_str("        _ => None,\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    out.push_str("pub fn block_to_item(block_id: u16) -> Option<i32> {\n");
    out.push_str("    match block_id {\n");

    for block in blocks.as_array().unwrap() {
        let block_id = block["defaultState"].as_i64().unwrap() as u16;
        let block_name = block["name"].as_str().unwrap();

        let item_id = items["entries"]
            .as_object()
            .unwrap()
            .iter()
            .find(|(ik, _)| ik.replace("minecraft:", "") == block_name)
            .and_then(|(_, iv)| iv["protocol_id"].as_i64())
            .unwrap_or(-1) as i32;

        if item_id != -1 {
            out.push_str(&format!("        {} => Some({}),\n", block_id, item_id));
        }
    }

    out.push_str("        _ => None,\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("item_to_block.rs");
    fs::write(out_path, out).unwrap();
}
