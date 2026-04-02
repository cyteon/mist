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

    pub spawned_at: std::time::Instant,
    pub dropped_by: Option<String>,

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
    // TODO: update position for players in range
    // TODO: spawn for players spawning or entering range that dont have it spawned already
    pub async fn tick(&mut self) -> anyhow::Result<()> {
        self.vy -= 0.04;

        self.vx *= 0.98;
        self.vy *= 0.98;
        self.vz *= 0.98;

        self.x += self.vx;
        self.y += self.vy;
        self.z += self.vz;
        
        let region = crate::world::get_region(self.x as i32 >> 9, self.z as i32 >> 9).await;
        let chunk = crate::world::get_chunk(&region, self.x as i32 >> 4, self.z as i32 >> 4).await;

        let lx = (self.x as i32 & 15) as u8;
        let lz = (self.z as i32 & 15) as u8;

        let surface = chunk.get_surface_y_below_point(lx, self.y as i32, lz) as i32;

        if self.y <= surface as f64 + 1.0 {
            self.y = surface as f64 + 1.0;
            self.vy = 0.0;
        }

        Ok(())
    }

    pub async fn broadcast_spawn(&mut self) {
        let players_owned = {
            let players = crate::server::state::play::PLAYERS.read().await;
            players.values().cloned().collect::<Vec<_>>()
        };

        for player in players_owned {
            let player_lock = player.lock().await;
            let distance_squared = (player_lock.x - self.x).powi(2) + (player_lock.y - self.y).powi(2) + (player_lock.z - self.z).powi(2);

            if distance_squared < 64.0 * 64.0 {
                let tx = crate::server::conn::PLAYER_SOCKET_MAP.read().await.get(&player_lock.uuid).cloned().unwrap();

                let mut buffer = Vec::new();
                crate::net::packets::clientbound::spawn_entity::send_spawn_entity(&mut buffer, self).await.unwrap();
                let _ = tx.send(buffer);

                let mut buffer = Vec::new();
                crate::net::packets::clientbound::set_entity_data::sent_set_entity_data(&mut buffer, self).await.unwrap();
                let _ = tx.send(buffer);
            }
        }
    }

    pub async fn broadcast_despawn(&self) {
        let players_owned = {
            let players = crate::server::state::play::PLAYERS.read().await;
            players.values().cloned().collect::<Vec<_>>()
        };

        for player in players_owned {
            let player_lock = player.lock().await;
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

pub fn spawn_item_drop(item_stack: super::items::ItemStack, dropped_by: Option<String>, x: f64, y: f64, z: f64) -> Entity {
    let entity = Entity {
        id: next_entity_id(),
        uuid: rand::random(),
        entity_type: EntityType::Item(item_stack),

        spawned_at: std::time::Instant::now(),
        dropped_by,

        x,
        y,
        z,

        yaw: 0.0,
        pitch: 0.0,

        vx: 0.0,
        vy: 0.0,
        vz: 0.0,
    };

    let entity_clone = entity.clone();
    tokio::spawn(async move {
        ENTITIES.write().await.insert(entity.id, entity_clone);
    });

    entity
}