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
    Item(super::items::ItemStack, std::time::Instant, Option<String>),
    // the player will manage their entity and update it themselves
    Player(String),
}

#[derive(Clone)]
pub struct Entity {
    pub id: i32,
    pub uuid: u128,
    pub entity_type: EntityType,

    pub x: f64,
    pub y: f64,
    pub z: f64,

    // used to determine when to send position updates, and with which packet
    pub last_x: f64,
    pub last_y: f64,
    pub last_z: f64,
    pub ticks_since_last_update: u32, // always send pos every 20 ticks

    pub yaw: f32,
    pub pitch: f32,

    pub last_yaw: f32,
    pub last_pitch: f32,

    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub on_ground: bool,
}

impl Entity {
    // TODO: update position for players in range
    pub async fn tick(&mut self) -> anyhow::Result<()> {
        if let EntityType::Player(_) = self.entity_type {
            let mut packet_buffer = Vec::new();

            if self.x == self.last_x && self.y == self.last_y && self.z == self.last_z {
                self.ticks_since_last_update += 1;

                if self.ticks_since_last_update >= 20 {
                    self.ticks_since_last_update = 0;
                    crate::net::packets::clientbound::entity_position_sync::send_entity_position_sync(&mut packet_buffer, self).await?;
                } else {
                    return Ok(());
                }
            } else {
                let dx = self.x - self.last_x;
                let dy = self.y - self.last_y;
                let dz = self.z - self.last_z;

                if dx.abs() >= 8.0 || dy.abs() >= 7.9 || dz.abs() >= 8.0 {
                    crate::net::packets::clientbound::entity_position_sync::send_entity_position_sync(&mut packet_buffer, self).await?;
                } else {
                    if self.yaw != self.last_yaw || self.pitch != self.last_pitch {
                        crate::net::packets::clientbound::move_entity_pos_rot::send_move_entity_pos_rot(&mut packet_buffer, self).await?;
                    } else {
                        crate::net::packets::clientbound::move_entity_pos::send_move_entity_pos(&mut packet_buffer, self).await?;
                    }
                }

                self.last_x = self.x;
                self.last_y = self.y;
                self.last_z = self.z;
                self.last_yaw = self.yaw;
                self.last_pitch = self.pitch;
            }

            let positions = super::player::PLAYER_POSITIONS.read().await;
            let socket_map = crate::server::conn::PLAYER_SOCKET_MAP.read().await;
            let view_distance_blocks = crate::config::SERVER_CONFIG.view_distance as f64 * 16.0;

            for (uuid, tx) in socket_map.iter() {
                if let EntityType::Player(my_uuid) = &self.entity_type {
                    if uuid == my_uuid { continue; }
                }

                if let Some(pos) = positions.get(uuid) {
                    let distance_squared = (pos.0 - self.x).powi(2) + (pos.2 - self.z).powi(2);

                    if distance_squared < view_distance_blocks * view_distance_blocks {
                        let _ = tx.send(packet_buffer.clone());
                    }
                }
            }

            return Ok(());
        } else if let EntityType::Item(..) = self.entity_type {
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
                self.on_ground = true;
            }
        }

        self.last_x = self.x;
        self.last_y = self.y;
        self.last_z = self.z;

        self.last_yaw = self.yaw;
        self.last_pitch = self.pitch;

        Ok(())
    }
}

pub fn spawn_item_drop(item_stack: super::items::ItemStack, dropped_by: Option<String>, x: f64, y: f64, z: f64) -> Entity {
    let entity = Entity {
        id: next_entity_id(),
        uuid: rand::random(),
        entity_type: EntityType::Item(item_stack, std::time::Instant::now(), dropped_by.clone()),

        x,
        y,
        z,

        last_x: x,
        last_y: y,
        last_z: z,
        ticks_since_last_update: 0,

        yaw: 0.0,
        pitch: 0.0,

        last_yaw: 0.0,
        last_pitch: 0.0,

        vx: 0.0,
        vy: 0.0,
        vz: 0.0,
        on_ground: false,
    };

    let entity_clone = entity.clone();
    tokio::spawn(async move {
        ENTITIES.write().await.insert(entity.id, entity_clone);
    });

    entity
}