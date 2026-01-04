use rsa::pkcs8::EncodePublicKey;
use sha1::{Sha1, Digest};
use num_bigint::BigInt;

use crate::RSA_PUBLIC_KEY;

pub struct PlayerData {
    pub username: String,
    pub textures: String,
    pub texture_signature: String,
}

fn generate_server_hash(shared_secret: &[u8]) -> String {
    let mut hasher = Sha1::new();
    
    hasher.update("".as_bytes());
    hasher.update(shared_secret);
    hasher.update(RSA_PUBLIC_KEY.to_public_key_der().unwrap().as_bytes());

    let hash_bytes = hasher.finalize();

    let hash = BigInt::from_signed_bytes_be(&hash_bytes);
    hash.to_str_radix(16)
}

pub async fn authenticate_player(username: &str, shared_secret: Vec<u8>) -> anyhow::Result<PlayerData> {
    let server_hash = generate_server_hash(shared_secret.as_slice());
    let url = format!(
        "https://sessionserver.mojang.com/session/minecraft/hasJoined?username={}&serverId={}",
        username,
        server_hash
    );

    let resp = reqwest::get(&url).await?;

    if resp.status().is_success() {
        let json: serde_json::Value = resp.json().await?;
        
        let username = json["name"].as_str().unwrap_or("").to_string();
        let textures = json["properties"][0]["value"].as_str().unwrap_or("").to_string();
        let texture_signature = json["properties"][0]["signature"].as_str().unwrap_or("").to_string();

        Ok(PlayerData { username, textures, texture_signature })
    } else {
        anyhow::bail!("Failed to authenticate player: HTTP {}", resp.status());
    }
}