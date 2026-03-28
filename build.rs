use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
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

    dbg!(tree);

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("packets.rs");
    fs::write(out_path, out).unwrap();
}