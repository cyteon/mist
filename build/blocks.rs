use std::env;
use std::fs;
use std::path::Path;

pub fn load_blocks() {
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

    out.push_str("\npub static BLOCKS: &[Block] = &[\n");

    for block in json.as_array().unwrap() {
        let name = block["name"].as_str().unwrap();
        let min_state_id = block["minStateId"].as_i64().unwrap() as u16;
        let default_state = block["defaultState"].as_i64().unwrap() as u16;

        out.push_str(&format!(
            "    Block {{ name: \"{}\", min_state_id: {}, default_state: {}, properties: &[",
            name, min_state_id, default_state
        ));

        for state in block["states"].as_array().unwrap() {
            let prop_name = state["name"].as_str().unwrap();

            let values: Vec<String> = if state["type"].as_str() == Some("bool") {
                vec!["true".to_string(), "false".to_string()]
            } else {
                state["values"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect()
            };

            out.push_str(&format!(
                "Property {{ name: \"{}\", values: &[{}] }}, ",
                prop_name,
                values
                    .iter()
                    .map(|v| format!("\"{}\"", v))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        out.push_str("] },\n");
    }

    out.push_str("];\n");

    out.push_str("\npub fn get_block_drops(block_id: u16) -> &'static [i32] {\n");
    out.push_str("    match block_id {\n");

    for block in json.as_array().unwrap() {
        let min_state_id = block["minStateId"].as_i64().unwrap() as u16;
        let max_state_id = block["maxStateId"].as_i64().unwrap() as u16;

        let drops = block["drops"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_i64().unwrap() as i32)
            .collect::<Vec<_>>();

        if max_state_id > min_state_id {
            out.push_str(&format!(
                "        {}..={} => &{:?},\n",
                min_state_id, max_state_id, drops
            ));
        } else {
            out.push_str(&format!("        {} => &{:?},\n", min_state_id, drops));
        }
    }

    out.push_str("        _ => &[],\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    out.push_str("\npub fn is_correct_tool(block_id: u16, item_id: i32) -> bool {\n");
    out.push_str("    match block_id {\n");

    for block in json.as_array().unwrap() {
        let min_state_id = block["minStateId"].as_i64().unwrap() as u16;
        let max_state_id = block["maxStateId"].as_i64().unwrap() as u16;

        if let Some(tools) = block["harvestTools"].as_object() {
            if !tools.is_empty() {
                let tool_ids: Vec<&str> = tools.keys().map(|k| k.as_str()).collect();

                if max_state_id > min_state_id {
                    out.push_str(&format!(
                        "        {}..={} => matches!(item_id, {}),\n",
                        min_state_id,
                        max_state_id,
                        tool_ids
                            .iter()
                            .map(|id| id.to_string())
                            .collect::<Vec<_>>()
                            .join(" | ")
                    ));
                } else {
                    out.push_str(&format!(
                        "        {} => matches!(item_id, {}),\n",
                        min_state_id,
                        tool_ids
                            .iter()
                            .map(|id| id.to_string())
                            .collect::<Vec<_>>()
                            .join(" | ")
                    ));
                }
            } else {
                out.push_str(&format!(
                    "        {}..={} => false,\n",
                    min_state_id, max_state_id
                ));
            }
        }
    }

    out.push_str("        _ => true,\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("blocks.rs");
    fs::write(out_path, out).unwrap();
}
