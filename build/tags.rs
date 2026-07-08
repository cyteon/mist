use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

fn read_tag_values(path: &Path) -> Vec<String> {
    let bytes = fs::read(path).expect("Failed to read tag file");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("Failed to parse tag file");

    json["values"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|value| value.as_str().map(|s| s.to_string()))
        .collect()
}

fn collect_raw_tags(dir: &Path) -> Vec<(String, Vec<String>)> {
    let mut tags = Vec::new();

    let Ok(entires) = fs::read_dir(dir) else {
        return tags;
    };

    let mut paths: Vec<_> = entires.flatten().map(|e| e.path()).collect();
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

                    tags.push((tag_name, read_tag_values(&subpath)));
                }
            }
        } else {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let tag_name = format!("minecraft:{}", file_name.strip_suffix(".json").unwrap());

            tags.push((tag_name, read_tag_values(&path)));
        }
    }

    tags
}

fn expand_nested_tags(raw: HashMap<String, Vec<String>>) -> HashMap<String, Vec<String>> {
    let mut raw = raw;

    // to protect against infinite loops
    for _ in 0..16 {
        let cloned = raw.clone();
        let mut changed = false;

        for values in raw.values_mut() {
            let mut expanded = Vec::new();

            for value in values.iter() {
                if let Some(refrence) = value.strip_prefix("#") {
                    changed = true;

                    if let Some(sub) = cloned.get(refrence) {
                        expanded.extend(sub.clone());
                    }
                } else {
                    expanded.push(value.clone());
                }
            }

            *values = expanded;
        }

        if !changed {
            break;
        }
    }

    raw
}

pub fn load_tags() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();

    let blocks_json_path = Path::new(&manifest).join("src/assets/blocks.json");
    let registries_json_path = Path::new(&manifest).join("src/assets/registries.json");
    let items_json_path = Path::new(&manifest).join("src/assets/items.json");

    let block_bytes = fs::read(&blocks_json_path).expect("Failed to read blocks.json");
    let registries_bytes = fs::read(&registries_json_path).expect("Failed to read registries.json");
    let items_bytes = fs::read(&items_json_path).expect("Failed to read items.json");

    let blocks: serde_json::Value =
        serde_json::from_slice(&block_bytes).expect("Failed to parse blocks.json");

    let registries: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&registries_bytes).expect("Failed to parse registries.json");

    let items: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&items_bytes).expect("Failed to parse items.json");

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

    let mut item_reg_ids: HashMap<String, i64> = HashMap::new();
    for (key, value) in items["entries"].as_object().unwrap().iter() {
        item_reg_ids.insert(key.clone(), value["protocol_id"].as_i64().unwrap());
    }

    let configs: &[(&str, &str, &HashMap<String, i64>)] = &[
        ("minecraft:block", "src/assets/tags/block", &block_reg_ids),
        (
            "minecraft:damage_type",
            "src/assets/tags/damage_type",
            &damage_type_reg_ids,
        ),
        ("minecraft:item", "src/assets/tags/item", &item_reg_ids),
    ];

    let mut out = String::new();
    out.push_str("pub static TAGS: &[(&str, &[(&str, &[i32])])] = &[\n");

    for (name, dir, reg_ids) in configs {
        let dir_path = Path::new(&manifest).join(dir);
        println!("cargo:rerun-if-changed={}", dir_path.display());

        let raw_tags = collect_raw_tags(&dir_path);
        let expanded_tags = expand_nested_tags(raw_tags.iter().cloned().collect());

        out.push_str(&format!("    (\"{}\", &[\n", name));

        for (name, _) in &raw_tags {
            let ids: Vec<i32> = expanded_tags
                .get(name)
                .unwrap_or(&Vec::new())
                .iter()
                .filter_map(|value| reg_ids.get(value).map(|&id| id as i32))
                .collect();

            out.push_str(&format!("        (\"{}\", &{:?}),\n", name, ids));
        }

        out.push_str("    ]),\n");
    }

    out.push_str("];\n");

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("tags.rs");
    fs::write(out_path, out).unwrap();
}
