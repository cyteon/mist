use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    load_packets();
    load_blocks();
    load_items();
    load_item_to_block();
}

fn load_packets() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let json_path = Path::new(&manifest).join("src/assets/packets.json");
    println!("cargo:rerun-if-changed={}", json_path.display());

    let bytes = fs::read(&json_path).expect("Failed to read packets.json");

    let json: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&bytes).expect("Failed to parse packets.json");
    
    let mut out = String::new();

    const PATHS: &[(&str, &str)] = &[
        ("handshake", "serverbound"),
        ("configuration", "clientbound"),
        ("configuration", "serverbound"),
        ("login", "clientbound"),
        ("login", "serverbound"),
        ("play", "clientbound"),
        ("play", "serverbound"),
        ("status", "clientbound"),
        ("status", "serverbound"),
    ];

    let mut tree: HashMap<&str, HashMap<&str, Vec<(String, i64)>>> = HashMap::new();
    for (state, direction) in PATHS {
        let obj = json[*state][*direction].as_object().unwrap();

        for (k, v) in obj {
            let protocol_id = v["protocol_id"].as_i64().unwrap();
            tree.entry(state).or_default().entry(direction).or_default().push((k.clone(), protocol_id));
        }
    }

    for (state, directions) in &tree {
        out.push_str(&format!("pub mod {} {{\n", state));

        for (direction, packets) in directions {
            out.push_str(&format!("    pub mod {} {{\n", direction));

            for (name, protocol_id) in packets {
                out.push_str(
                    &format!(
                        "        pub const {}: u32 = {};\n", 
                        name.to_uppercase().replace("MINECRAFT:", "").replace("/", "_"),
                        protocol_id
                    )
                );
            }

            out.push_str("    }\n");
        }

        out.push_str("}\n");
    }

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("packets.rs");
    fs::write(out_path, out).unwrap();
}

fn load_blocks() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let json_path = Path::new(&manifest).join("src/assets/blocks.json");
    println!("cargo:rerun-if-changed={}", json_path.display());

    let bytes = fs::read(&json_path).expect("Failed to read blocks.json");

    let json: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&bytes).expect("Failed to parse blocks.json");
    
    let mut out = String::new();

    for (key, block) in json {
        let default_state = block["states"]
            .as_array()
            .unwrap()
            .iter()
            .find(|state| state["default"].as_bool().unwrap_or(false))
            .expect("Block is missing a default state");

        out.push_str(
            &format!(
                "pub const {}: u16 = {};\n",
                key.to_uppercase().replace("MINECRAFT:", "").replace("/", "_"),
                default_state["id"].as_u64().unwrap()
            )
        );
    } 

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("blocks.rs");
    fs::write(out_path, out).unwrap();
}

fn load_items() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let json_path = Path::new(&manifest).join("src/assets/items.json");
    println!("cargo:rerun-if-changed={}", json_path.display());

    let bytes = fs::read(&json_path).expect("Failed to read items.json");

    let json: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&bytes).expect("Failed to parse items.json");

    let mut out = String::new();

    for (key, item) in json["entries"].as_object().unwrap() {
        out.push_str(
            &format!(
                "pub const {}: i32 = {};\n",
                key.to_uppercase().replace("MINECRAFT:", "").replace("/", "_"),
                item["protocol_id"].as_u64().unwrap()
            )
        );
    }

    out.push_str("pub fn get_item_id(name: &str) -> i32 {\n");
    out.push_str("    match name {\n");

    for (key, item) in json["entries"].as_object().unwrap() {
        out.push_str(
            &format!(
                "        \"{}\" => {},\n",
                key,
                item["protocol_id"].as_u64().unwrap()
            )
        );
    }

    out.push_str("        _ => 0,\n"); // default is air
    out.push_str("    }\n");
    out.push_str("}\n");

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("items.rs");
    fs::write(out_path, out).unwrap();
}

fn load_item_to_block() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let blocks_json_path = Path::new(&manifest).join("src/assets/blocks.json");
    let items_json_path = Path::new(&manifest).join("src/assets/items.json");

    let block_bytes = fs::read(&blocks_json_path).expect("Failed to read blocks.json");
    let item_bytes = fs::read(&items_json_path).expect("Failed to read items.json");

    let blocks: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&block_bytes).expect("Failed to parse blocks.json");
    let items: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&item_bytes).expect("Failed to parse items.json");
    
    let mut out = String::new();
    out.push_str("pub fn item_to_block(item_id: i32) -> Option<u16> {\n");
    out.push_str("    match item_id {\n");

    for (ik, iv) in items["entries"].as_object().unwrap() {
        if let Some(block) = blocks.get(ik) {
            let default_state = block["states"]
                .as_array()
                .unwrap()
                .iter()
                .find(|state| state["default"].as_bool().unwrap_or(false))
                .expect("Block is missing a default state");

            out.push_str(
                &format!(
                    "        {} => Some({}),\n",
                    iv["protocol_id"].as_u64().unwrap(),
                    default_state["id"].as_u64().unwrap()
                )
            );
        }
    }

    out.push_str("        _ => None,\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    out.push_str("pub fn block_to_item_id(block_id: i32) -> Option<u16> {\n");
    out.push_str("    match block_id {\n");

    for (ik, iv) in items["entries"].as_object().unwrap() {
        if let Some(block) = blocks.get(ik) {
            let default_state = block["states"]
                .as_array()
                .unwrap()
                .iter()
                .find(|state| state["default"].as_bool().unwrap_or(false))
                .expect("Block is missing a default state");

            out.push_str(
                &format!(
                    "        {} => Some({}),\n",
                    default_state["id"].as_u64().unwrap(),
                    iv["protocol_id"].as_u64().unwrap()
                )
            );
        }
    }

    out.push_str("        _ => None,\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("item_to_block.rs");
    fs::write(out_path, out).unwrap();
}