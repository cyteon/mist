use std::collections::HashMap;
use once_cell::sync::Lazy;

pub struct Block {
    pub id: u16,
}

pub static BLOCKS: Lazy<HashMap<String, Block>> = Lazy::new(|| {
    let bytes = include_bytes!("../assets/blocks.json");
    let json: HashMap<String, RawBlockData> =
        serde_json::from_slice(bytes).expect("Failed to parse blocks.json");

    let mut hasmap = HashMap::new();

    for (key, block) in json {
        let default_state = block
            .states
            .iter()
            .find(|state| state.default)
            .expect("Block is missing a default state");

        hasmap.insert(
            key.clone(),
            Block {
                id: default_state.id,
            },
        );
    }

    return hasmap;
});

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RawBlockData {
    pub definition: serde_json::Value,
    #[serde(default)]
    pub properties: HashMap<String, Vec<String>>,
    pub states: Vec<RawBlockState>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RawBlockState {
    pub id: u16,
    #[serde(default)]
    pub default: bool,
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

pub fn get(key: &str) -> Option<&'static Block> {
    BLOCKS.get(key)
}