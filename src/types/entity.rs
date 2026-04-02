use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicI32, Ordering};
use tokio::sync::RwLock;
use std::collections::HashMap;

pub static ENTITIES: Lazy<RwLock<HashMap<i32, Entity>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

static NEXT_ENTITY_ID: AtomicI32 = AtomicI32::new(1);

pub fn next_entity_id() -> i32 {
    NEXT_ENTITY_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone)]
pub enum EntityType {
    Item(super::items::ItemStack),
}

#[derive(Clone)]
pub struct Entity {
    pub id: i32,
    pub uuid: u128,
    pub entity_type: EntityType,

    pub x: f64,
    pub y: f64,
    pub z: f64,

    pub yaw: f32,
    pub pitch: f32,

    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
}

impl Entity {
    pub async fn broadcast_spawn(&mut self) {
        let players_owned = {
            let players = crate::server::state::play::PLAYERS.read().await;
            players.values().cloned().collect::<Vec<_>>()
        };

        println!("Broadcasting spawn of entity {} to {} players", self.id, players_owned.len());

        for player in players_owned {
            println!("Checking distance from player {} to entity {}", player.lock().await.username, self.id);

            let mut player_lock = player.lock().await;
            let distance_squared = (player_lock.x - self.x).powi(2) + (player_lock.y - self.y).powi(2) + (player_lock.z - self.z).powi(2);

            println!("Distance from player {} to entity {}: {}", player_lock.username, self.id, distance_squared.sqrt());

            if distance_squared < 64.0 * 64.0 {
                let tx = crate::server::conn::PLAYER_SOCKET_MAP.read().await.get(&player_lock.uuid).cloned().unwrap();

                let mut buffer = Vec::new();
                crate::net::packets::clientbound::spawn_entity::send_spawn_entity(&mut buffer, self).await.unwrap();
                let _ = tx.send(buffer);

                let mut buffer = Vec::new();
                crate::net::packets::clientbound::set_entity_data::sent_set_entity_data(&mut buffer, self).await.unwrap();
                let _ = tx.send(buffer);

                println!("Broadcasted spawn of entity {} to player {}", self.id, player_lock.username);
            }
        }
    }

    pub async fn broadcast_despawn(&self) {
        let players_owned = {
            let players = crate::server::state::play::PLAYERS.read().await;
            players.values().cloned().collect::<Vec<_>>()
        };

        for player in players_owned {
            let mut player_lock = player.lock().await;
            let distance_squared = (player_lock.x - self.x).powi(2) + (player_lock.y - self.y).powi(2) + (player_lock.z - self.z).powi(2);

            if distance_squared < 64.0 * 64.0 {
                let tx = crate::server::conn::PLAYER_SOCKET_MAP.read().await.get(&player_lock.uuid).cloned().unwrap();

                let mut buffer = Vec::new();
                crate::net::packets::clientbound::remove_entities::send_remove_entities(&mut buffer, vec![self.id]).await.unwrap();
                let _ = tx.send(buffer);
            }
        }
    }
}

pub fn spawn_item_drop(item_stack: super::items::ItemStack, x: f64, y: f64, z: f64) -> Entity {
    let entity = Entity {
        id: next_entity_id(),
        uuid: rand::random(),
        entity_type: EntityType::Item(item_stack),

        x,
        y,
        z,

        yaw: 0.0,
        pitch: 0.0,

        vx: 0.0,
        vy: 1.0,
        vz: 0.0,
    };

    let entity_clone = entity.clone();
    tokio::spawn(async move {
        ENTITIES.write().await.insert(entity.id, entity_clone);
    });

    return entity;
}