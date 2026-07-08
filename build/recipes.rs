use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

pub fn load_recipes() {
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

pub fn load_furnace_recipes() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let json_path = Path::new(&manifest).join("src/assets/furnace_recipes.json");
    println!("cargo:rerun-if-changed={}", json_path.display());

    let recipes_bytes = fs::read(&json_path).expect("Failed to read furnace_recipes.json");
    let recipes: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&recipes_bytes).expect("Failed to parse furnace_recipes.json");

    let items_bytes = fs::read(Path::new(&manifest).join("src/assets/items.json"))
        .expect("Failed to read items.json");
    let items: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&items_bytes).expect("Failed to parse items.json");

    let mut out = String::new();

    for (type_, recipes) in recipes {
        let stripped_type = type_.strip_prefix("minecraft:").unwrap_or(&type_);

        let mut function = String::new();

        function.push_str(&format!(
            "pub fn get_{}_recipe(item: i32) -> Option<i32> {{\n",
            stripped_type
        ));

        function.push_str("    match item {\n");

        for (input, output) in recipes.as_object().unwrap() {
            if input.starts_with('#') {
                continue; // NONONONONONOO
            }

            let input_id = items["entries"][input]["protocol_id"].as_i64().unwrap() as i32;
            let output_id = items["entries"][output.as_str().unwrap()]["protocol_id"]
                .as_i64()
                .unwrap() as i32;

            function.push_str(&format!("        {} => Some({}),\n", input_id, output_id));
        }

        function.push_str("        _ => None,\n");
        function.push_str("    }\n");
        function.push_str("}\n");

        out.push_str(&function);
    }

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("furnace_recipes.rs");
    fs::write(out_path, out).unwrap();
}
