use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    load_packets();
    load_blocks();
    load_items();
    load_item_to_block();
    load_recipes();
    encode_item_components();
    load_tags();
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
            tree.entry(state)
                .or_default()
                .entry(direction)
                .or_default()
                .push((k.clone(), protocol_id));
        }
    }

    for (state, directions) in &tree {
        out.push_str(&format!("pub mod {} {{\n", state));

        for (direction, packets) in directions {
            out.push_str(&format!("    pub mod {} {{\n", direction));

            for (name, protocol_id) in packets {
                out.push_str(&format!(
                    "        pub const {}: u32 = {};\n",
                    name.to_uppercase()
                        .replace("MINECRAFT:", "")
                        .replace("/", "_"),
                    protocol_id
                ));
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

    let json: serde_json::Value =
        serde_json::from_slice(&bytes).expect("Failed to parse blocks.json");

    let mut out = String::new();

    for block in json.as_array().unwrap() {
        out.push_str(&format!(
            "pub const {}: u16 = {};\n",
            block["name"].as_str().unwrap().to_uppercase(),
            block["defaultState"].as_i64().unwrap()
        ));
    }

    out.push_str("\npub fn get_block_drops(block_id: u16) -> &'static [i32] {\n");
    out.push_str("    match block_id {\n");

    for block in json.as_array().unwrap() {
        let block_id = block["defaultState"].as_i64().unwrap();
        let drops = block["drops"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_i64().unwrap() as i32)
            .collect::<Vec<_>>();

        out.push_str(&format!("        {} => &{:?},\n", block_id, drops));
    }

    out.push_str("        _ => &[],\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    out.push_str("\npub fn is_correct_tool(block_id: u16, item_id: i32) -> bool {\n");
    out.push_str("    match block_id {\n");

    for block in json.as_array().unwrap() {
        let block_id = block["defaultState"].as_i64().unwrap();

        if let Some(tools) = block["harvestTools"].as_object() {
            if !tools.is_empty() {
                let tool_ids: Vec<&str> = tools.keys().map(|k| k.as_str()).collect();

                out.push_str(&format!(
                    "        {} => matches!(item_id, {}),\n",
                    block_id,
                    tool_ids
                        .iter()
                        .map(|id| format!("{}", id))
                        .collect::<Vec<_>>()
                        .join(" | ")
                ));
            } else {
                out.push_str(&format!("        {} => false,\n", block_id));
            }
        }
    }

    out.push_str("        _ => true,\n");
    out.push_str("    }\n");
    out.push_str("}\n");

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

fn load_item_to_block() {
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

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("item_to_block.rs");
    fs::write(out_path, out).unwrap();
}

fn load_recipes() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let json_path = Path::new(&manifest).join("src/assets/recipes.json");
    println!("cargo:rerun-if-changed={}", json_path.display());

    let bytes = fs::read(&json_path).expect("Failed to read recipes.json");
    let json: serde_json::Value =
        serde_json::from_slice(&bytes).expect("Failed to parse recipes.json");

    let mut out = String::new();
    out.push_str("pub static RECIPES: &[Recipe] = &[\n");

    for (_, recipes) in json.as_object().unwrap() {
        for recipe in recipes.as_array().unwrap() {
            let result_id = recipe["result"]["id"].as_i64().unwrap();
            let result_count = recipe["result"]["count"].as_i64().unwrap();

            if let Some(in_shape) = recipe.get("inShape") {
                let rows = in_shape.as_array().unwrap();
                let height = rows.len() as i32;
                let width = rows[0].as_array().unwrap().len() as i32;

                let mut pattern = [0i32; 9];

                for (r, row) in rows.iter().enumerate() {
                    for (c, item) in row.as_array().unwrap().iter().enumerate() {
                        pattern[r * 3 + c] = if item.is_null() {
                            0
                        } else {
                            item.as_i64().unwrap() as i32
                        };
                    }
                }

                out.push_str(&format!(
                    "    Recipe::Shaped(ShapedRecipe {{ pattern: {:?}, width: {}, height: {}, result_id: {}, result_count: {} }}),\n",
                    pattern, width, height, result_id, result_count
                ));
            } else if let Some(ingredients) = recipe.get("ingredients") {
                let items: Vec<i32> = ingredients
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|item| item.as_i64().unwrap() as i32)
                    .collect();

                out.push_str(&format!(
                    "    Recipe::Shapeless(ShapelessRecipe {{ ingredients: &{:?}, result_id: {}, result_count: {} }}),\n",
                    items, result_id, result_count
                ));
            }
        }
    }

    out.push_str("];\n");

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("recipes.rs");
    fs::write(out_path, out).unwrap();
}

fn encode_item_components() {
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

fn load_tags() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let blocks_json_path = Path::new(&manifest).join("src/assets/blocks.json");
    let registries_json_path = Path::new(&manifest).join("src/assets/registries.json");

    let block_bytes = fs::read(&blocks_json_path).expect("Failed to read blocks.json");
    let registries_bytes = fs::read(&registries_json_path).expect("Failed to read registries.json");

    let blocks: serde_json::Value =
        serde_json::from_slice(&block_bytes).expect("Failed to parse blocks.json");

    let registries: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&registries_bytes).expect("Failed to parse registries.json");

    let mut block_reg_ids: HashMap<String, i64> = HashMap::new();
    for (idx, block) in blocks.as_array().unwrap().iter().enumerate() {
        let name = block["name"].as_str().unwrap();

        block_reg_ids.insert(format!("minecraft:{}", name), idx as i64);
    }

    let mut damage_type_reg_ids: HashMap<String, i64> = HashMap::new();
    for (idx, key) in registries["damage_type"]
        .as_object()
        .unwrap()
        .keys()
        .enumerate()
    {
        damage_type_reg_ids.insert(format!("minecraft:{}", key), idx as i64);
    }

    let mut banner_pattern_ids: HashMap<String, i64> = HashMap::new();
    for (idx, key) in registries["banner_pattern"]
        .as_object()
        .unwrap()
        .keys()
        .enumerate()
    {
        banner_pattern_ids.insert(format!("minecraft:{}", key), idx as i64);
    }

    let configs: &[(&str, &str, &HashMap<String, i64>)] = &[
        ("minecraft:block", "src/assets/tags/block", &block_reg_ids),
        (
            "minecraft:damage_type",
            "src/assets/tags/damage_type",
            &damage_type_reg_ids,
        ),
        (
            "minecraft:banner_pattern",
            "src/assets/tags/banner_pattern",
            &banner_pattern_ids,
        ),
    ];

    let mut out = String::new();
    out.push_str("pub static TAGS: &[(&str, &[(&str, &[i32])])] = &[\n");

    for (name, dir, reg_ids) in configs {
        let dir_path = Path::new(&manifest).join(dir);
        println!("cargo:rerun-if-changed={}", dir_path.display());

        let mut tags = Vec::new();

        if let Ok(entries) = fs::read_dir(&dir_path) {
            let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
            paths.sort();

            for path in paths {
                if path.is_dir() {
                    let subdir_name = path.file_name().unwrap().to_str().unwrap();

                    if let Ok(subentries) = fs::read_dir(&path) {
                        let mut subpaths: Vec<_> = subentries.flatten().map(|e| e.path()).collect();
                        subpaths.sort();

                        for subpath in subpaths {
                            let file_name = subpath.file_name().unwrap().to_str().unwrap();
                            let tag_name = format!(
                                "minecraft:{}/{}",
                                subdir_name,
                                file_name.strip_suffix(".json").unwrap()
                            );

                            let bytes = fs::read(&subpath).expect("Failed to read tag file");
                            let json: serde_json::Value =
                                serde_json::from_slice(&bytes).expect("Failed to parse tag file");

                            let mut ids = Vec::new();

                            for value in json["values"].as_array().unwrap() {
                                let name = value.as_str().unwrap_or("");

                                if name.starts_with('#') {
                                    // todo: nested tags
                                    continue;
                                }

                                if let Some(&id) = reg_ids.get(name) {
                                    ids.push(id as i32);
                                }
                            }

                            tags.push((tag_name, ids));
                        }
                    }
                } else {
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    let tag_name =
                        format!("minecraft:{}", file_name.strip_suffix(".json").unwrap());

                    let bytes = fs::read(&path).expect("Failed to read tag file");
                    let json: serde_json::Value =
                        serde_json::from_slice(&bytes).expect("Failed to parse tag file");

                    let mut ids = Vec::new();

                    for value in json["values"].as_array().unwrap() {
                        let name = value.as_str().unwrap_or("");

                        if name.starts_with('#') {
                            continue;
                        }

                        if let Some(&id) = reg_ids.get(name) {
                            ids.push(id as i32);
                        }
                    }

                    tags.push((tag_name, ids));
                }
            }
        }

        out.push_str(&format!("    (\"{}\", &[\n", name));

        for (tag_name, ids) in tags {
            out.push_str(&format!("        (\"{}\", &{:?}),\n", tag_name, ids));
        }

        out.push_str("    ]),\n");
    }

    out.push_str("];\n");

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("tags.rs");
    fs::write(out_path, out).unwrap();
}

// stuff for encoding stuff

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
                KNOWN_TAGS.iter().any(|tag| *tag == blocks)
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
