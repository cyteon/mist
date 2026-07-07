use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use tokio::sync::RwLock;

use crate::net::packets::clientbound::entity_position_sync::send_entity_position_sync;
use crate::net::packets::clientbound::set_entity_data::send_set_entity_data;
use crate::types::player::broadcast_packet;

pub static ENTITIES: Lazy<RwLock<HashMap<i32, Entity>>> = Lazy::new(|| RwLock::new(HashMap::new()));

static NEXT_ENTITY_ID: AtomicI32 = AtomicI32::new(1);

pub fn next_entity_id() -> i32 {
    NEXT_ENTITY_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone)]
pub struct ItemEntity {
    pub item_stack: super::items::ItemStack,
    pub dropped_at: std::time::Instant,
    pub dropped_by: Option<String>,
}

#[derive(Clone)]
pub struct PlayerEntity {
    pub uuid: String,
    pub health: f32,
}

#[derive(Clone)]
pub enum EntityType {
    Item(ItemEntity),
    // the player will manage their entity and update it themselves
    Player(PlayerEntity),
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
    pub async fn tick(&mut self) -> anyhow::Result<()> {
        if let EntityType::Player(player_entity) = &self.entity_type {
            // TODO: sync pose (swimming, etc)

            let mut packet_buffer = Vec::new();

            self.ticks_since_last_update += 1;
            let force_sync = self.ticks_since_last_update >= 20;

            if self.x == self.last_x && self.y == self.last_y && self.z == self.last_z {
                if force_sync {
                    self.ticks_since_last_update = 0;
                    crate::net::packets::clientbound::entity_position_sync::send_entity_position_sync(&mut packet_buffer, self).await?;
                } else if self.yaw != self.last_yaw || self.pitch != self.last_pitch {
                    crate::net::packets::clientbound::move_entity_rot::send_move_entity_rot(
                        &mut packet_buffer,
                        self,
                    )
                    .await?;
                } else {
                    return Ok(());
                }
            } else {
                let dx = self.x - self.last_x;
                let dy = self.y - self.last_y;
                let dz = self.z - self.last_z;

                if force_sync || dx.abs() >= 8.0 || dy.abs() >= 7.9 || dz.abs() >= 8.0 {
                    send_entity_position_sync(&mut packet_buffer, self).await?;
                } else if self.yaw != self.last_yaw || self.pitch != self.last_pitch {
                    crate::net::packets::clientbound::move_entity_pos_rot::send_move_entity_pos_rot(&mut packet_buffer, self).await?;
                } else {
                    crate::net::packets::clientbound::move_entity_pos::send_move_entity_pos(
                        &mut packet_buffer,
                        self,
                    )
                    .await?;
                }
            }

            broadcast_packet(
                packet_buffer,
                (self.x, self.y, self.z),
                Some(player_entity.uuid.clone()),
            )
            .await?;

            if self.yaw != self.last_yaw {
                let mut buffer = Vec::new();
                crate::net::packets::clientbound::rotate_head::send_rotate_head(
                    &mut buffer,
                    self.id,
                    self.yaw,
                )
                .await?;

                broadcast_packet(
                    buffer,
                    (self.x, self.y, self.z),
                    Some(player_entity.uuid.clone()),
                )
                .await?;
            }

            self.last_x = self.x;
            self.last_y = self.y;
            self.last_z = self.z;
            self.last_yaw = self.yaw;
            self.last_pitch = self.pitch;

            return Ok(());
        } else if let EntityType::Item(_) = self.entity_type {
            self.vy -= 0.04;

            self.vx *= if self.on_ground { 0.6 * 0.98 } else { 0.98 };
            self.vy *= 0.98;
            self.vz *= if self.on_ground { 0.6 * 0.98 } else { 0.98 };

            self.x += self.vx;
            self.y += self.vy;
            self.z += self.vz;

            let region = crate::world::get_region(self.x as i32 >> 9, self.z as i32 >> 9).await;
            let chunk =
                crate::world::get_chunk(&region, self.x as i32 >> 4, self.z as i32 >> 4).await;

            let lx = (self.x as i32 & 15) as u8;
            let lz = (self.z as i32 & 15) as u8;

            let surface = chunk.get_surface_y_below_point(lx, self.y as i32, lz) as i32;

            if self.y <= surface as f64 + 1.0 {
                self.y = surface as f64 + 1.0;
                self.vy = 0.0;
                self.on_ground = true;
            } else {
                self.on_ground = false;
            }

            let mut packet_buffer = Vec::new();

            self.ticks_since_last_update += 1;
            let force_sync = self.ticks_since_last_update >= 20;

            if self.x == self.last_x && self.y == self.last_y && self.z == self.last_z {
                if force_sync {
                    self.ticks_since_last_update = 0;
                    send_entity_position_sync(&mut packet_buffer, self).await?;
                }
            } else {
                let dx = self.x - self.last_x;
                let dy = self.y - self.last_y;
                let dz = self.z - self.last_z;

                if force_sync || dx.abs() >= 8.0 || dy.abs() >= 7.9 || dz.abs() >= 8.0 {
                    send_entity_position_sync(&mut packet_buffer, self).await?;
                } else {
                    crate::net::packets::clientbound::move_entity_pos::send_move_entity_pos(
                        &mut packet_buffer,
                        self,
                    )
                    .await?;
                }
            }

            broadcast_packet(packet_buffer, (self.x, self.y, self.z), None).await?;
        }

        self.last_x = self.x;
        self.last_y = self.y;
        self.last_z = self.z;

        self.last_yaw = self.yaw;
        self.last_pitch = self.pitch;

        Ok(())
    }

    pub async fn send_hand_swing(&self, main_hand: bool) -> anyhow::Result<()> {
        if let EntityType::Player(player_entity) = &self.entity_type {
            let mut buffer = Vec::new();

            let animation = match main_hand {
                true => crate::net::packets::clientbound::animate::Animation::SwingMainArm,
                false => crate::net::packets::clientbound::animate::Animation::SwingOffArm,
            };

            crate::net::packets::clientbound::animate::send_animate(
                &mut buffer,
                self.id,
                animation,
            )
            .await?;

            broadcast_packet(
                buffer,
                (self.x, self.y, self.z),
                Some(player_entity.uuid.clone()),
            )
            .await?;
        } else {
            anyhow::bail!("Only player entities can swing their arms");
        }

        Ok(())
    }

    pub async fn sync_entity_data(&self) -> anyhow::Result<()> {
        let mut buffer = Vec::new();
        send_set_entity_data(&mut buffer, self).await?;

        if let EntityType::Player(player_entity) = &self.entity_type {
            broadcast_packet(
                buffer,
                (self.x, self.y, self.z),
                Some(player_entity.uuid.clone()),
            )
            .await?;
        } else {
            broadcast_packet(buffer, (self.x, self.y, self.z), None).await?;
        }

        Ok(())
    }
}

pub fn spawn_item_drop(
    item_stack: super::items::ItemStack,
    dropped_by: Option<String>,
    (x, y, z): (f64, f64, f64),
    (vx, vy, vz): (f64, f64, f64),
) -> Entity {
    let entity = Entity {
        id: next_entity_id(),
        uuid: rand::random(),
        entity_type: EntityType::Item(ItemEntity {
            item_stack,
            dropped_at: std::time::Instant::now(),
            dropped_by,
        }),

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

        vx,
        vy,
        vz,

        on_ground: false,
    };

    let entity_clone = entity.clone();
    tokio::spawn(async move {
        ENTITIES.write().await.insert(entity.id, entity_clone);
    });

    entity
}
