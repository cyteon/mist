use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

pub fn load_food_data() {
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
    // nutrition, saturation, can always eat
    out.push_str("pub fn get_food_data(item_id: i32) -> Option<(i32, f32, bool)> {\n");
    out.push_str("    match item_id {\n");

    for (name, data) in item_data {
        if let Some(food) = data["components"].get("minecraft:food") {
            let protocol_id = item_protocol["entries"][&name]["protocol_id"]
                .as_i64()
                .unwrap() as i32;

            let nutrition = food["nutrition"].as_i64().unwrap() as i32;
            let saturation = food["saturation"].as_f64().unwrap_or(0.0) as f32;
            let can_always_eat = food["can_always_eat"].as_bool().unwrap_or(false);

            out.push_str(&format!(
                "        {} => Some(({}, {}f32, {})),\n",
                protocol_id, nutrition, saturation, can_always_eat
            ));
        }
    }

    out.push_str("        _ => None,\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("food_data.rs");
    fs::write(out_path, out).unwrap();
}
