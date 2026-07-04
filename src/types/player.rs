use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::atomic::Ordering};
use tokio::sync::RwLock;

use crate::{
    net::packets::clientbound::{
        container_set_content::send_container_set_content,
        container_set_slot::send_container_set_slot,
        damage_event::send_damage_event,
        entity_event::send_entity_event,
        game_event::{GameEvent, send_game_event},
        level_chunk_with_light::send_level_chunk_with_light,
        player_info_update::{PlayerAction, send_player_info_update},
        remove_entities::send_remove_entities,
        respawn::send_respawn,
        set_center_chunk::send_set_center_chunk,
        set_entity_data::sent_set_entity_data,
        set_health::send_set_health,
        spawn_entity::send_spawn_entity,
        sync_player_position::send_sync_player_position,
        system_chat_message::send_system_chat_message,
        update_time::send_update_time,
    },
    types::items::get_food_data,
    world::{get_chunk, get_region},
};

pub static PLAYER_POSITIONS: Lazy<RwLock<HashMap<String, (f64, f64, f64)>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Clone)]
pub struct PlayerMovement {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jumping: bool,
    pub sneaking: bool,
    pub sprinting: bool,
}

#[derive(PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize, Default)]
pub enum Gamemode {
    #[default]
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Clone, Copy)]
pub enum WindowType {
    CraftingTable([Option<crate::types::items::ItemStack>; 10]),
}

#[derive(Clone)]
pub struct Player {
    pub id: i32,
    pub uuid: String,
    pub username: String,

    pub current_window: Option<WindowType>,
    pub window_id: i32,

    pub inventory: [Option<super::items::ItemStack>; 46],
    pub carried_item: Option<super::items::ItemStack>,
    pub current_slot: i16,

    pub is_op: bool,
    pub gamemode: Gamemode,

    pub health: f32,
    pub hunger: i32,
    pub saturation: f32,
    pub stats_changed: bool,

    pub eating_ticks_left: u32,
    pub eating_slot: i16,

    pub exhaustion: f32, // resets to 0 and drains hunger when it hits 4
    pub regen_timer: u32,
    pub starvation_timer: u32,
    // todo: food poisoning
    pub dead: bool,
    pub ignore_fall_for_ticks: u32,

    pub shared_secret: Option<Vec<u8>>,
    pub textures: Option<String>,
    pub texture_signature: Option<String>,

    pub x: f64,
    pub y: f64,
    pub z: f64,

    // used to determine what chunks to send and fall damage
    pub last_x: f64,
    pub last_z: f64,

    pub vx: f64,
    pub vy: f64,
    pub vz: f64,

    // only for fall damage calc
    pub server_vy: f64,
    pub jump_applied: bool,

    pub on_ground: bool,
    pub flying: bool,
    pub fall_distance: f64,

    pub yaw: f32,
    pub pitch: f32,

    pub movement: PlayerMovement,

    pub initial_sync_done: bool,
    pub chunks_loaded: bool,
    pub chat_index: i32,
    pub loaded_entities: Vec<i32>,
    pub ticks_since_time_update: u32,
}

// TODO: hunger
// TODO: health regen

impl Player {
    const EX_SPRINT: f32 = 0.1;
    const EX_JUMP: f32 = 0.05;
    const EX_SPRINT_JUMP: f32 = 0.2;
    const EX_SWIM: f32 = 0.01;
    const EX_ATTACK: f32 = 0.1;
    const EX_DAMAGE_TAKEN: f32 = 0.1;
    const EX_HEART_REGEN: f32 = 6.0;

    pub async fn new(uuid: String, username: String) -> Self {
        let mut player = Player {
            id: super::entity::next_entity_id(),
            uuid,
            username: username.clone(),

            current_window: None,
            window_id: 0,

            inventory: [None; 46],
            carried_item: None,
            current_slot: 0,

            is_op: false,
            gamemode: match crate::config::SERVER_CONFIG.default_gamemode.as_str() {
                "survival" => Gamemode::Survival,
                "creative" => Gamemode::Creative,
                "adventure" => Gamemode::Adventure,
                "spectator" => Gamemode::Spectator,
                _ => {
                    crate::log::log(
                        fancy_log::LogLevel::Warn,
                        &format!(
                            "Invalid default gamemode in config: {}, defaulting to survival",
                            crate::config::SERVER_CONFIG.default_gamemode
                        ),
                    );
                    Gamemode::Survival
                }
            },

            health: 20.0,
            hunger: 20,
            saturation: 5.0,
            stats_changed: false,

            eating_ticks_left: 0,
            eating_slot: 0,

            exhaustion: 0.0,
            regen_timer: 0,
            starvation_timer: 0,

            shared_secret: None,
            textures: None,
            texture_signature: None,

            x: 0.0,
            y: 0.0,
            z: 0.0,

            last_x: 0.0,
            last_z: 0.0,

            vx: 0.0,
            vy: 0.0,
            vz: 0.0,

            server_vy: 0.0,
            jump_applied: false,

            on_ground: true,
            flying: false,
            fall_distance: 0.0,
            dead: false,
            ignore_fall_for_ticks: 0,

            yaw: 0.0,
            pitch: 0.0,

            movement: PlayerMovement {
                forward: false,
                backward: false,
                left: false,
                right: false,
                jumping: false,
                sneaking: false,
                sprinting: false,
            },

            initial_sync_done: false,
            chat_index: -1,
            chunks_loaded: false,
            loaded_entities: Vec::new(),
            ticks_since_time_update: 20,
        };

        let player_save = crate::server::save::load_player(&player.uuid);

        if let Some(player_save) = player_save {
            player.inventory = player_save.inventory.try_into().unwrap_or([None; 46]);

            player.is_op = player_save.is_op;
            player.gamemode = player_save.gamemode;

            player.health = player_save.health;
            player.hunger = player_save.hunger;
            player.saturation = player_save.saturation;
            player.dead = player_save.dead;

            player.x = player_save.x;
            player.y = player_save.y;
            player.z = player_save.z;

            player.vx = player_save.vx;
            player.vy = player_save.vy;
            player.vz = player_save.vz;

            player.yaw = player_save.yaw;
            player.pitch = player_save.pitch;

            crate::log::log(
                fancy_log::LogLevel::Info,
                &format!("Loaded save for player {}", username),
            );
        } else {
            player.inventory[36] = Some(super::items::ItemStack {
                item_id: super::items::GRASS_BLOCK,
                count: 64,
            });
            player.inventory[37] = Some(super::items::ItemStack {
                item_id: super::items::DIRT,
                count: 64,
            });
            player.inventory[38] = Some(super::items::ItemStack {
                item_id: super::items::STONE,
                count: 64,
            });
        }

        let entity = super::entity::Entity {
            id: player.id,
            uuid: {
                let uuid = player.uuid.replace("-", "");
                u128::from_be_bytes(hex::decode(uuid).unwrap().try_into().unwrap())
            },
            entity_type: super::entity::EntityType::Player(super::entity::PlayerEntity {
                uuid: player.uuid.clone(),
            }),

            x: player.x,
            y: player.y,
            z: player.z,

            last_x: player.x,
            last_y: player.y,
            last_z: player.z,
            ticks_since_last_update: 0,

            yaw: player.yaw,
            pitch: player.pitch,

            last_yaw: player.yaw,
            last_pitch: player.pitch,

            vx: player.vx,
            vy: player.vy,
            vz: player.vz,
            on_ground: player.on_ground,
        };

        crate::types::entity::ENTITIES
            .write()
            .await
            .insert(entity.id, entity);

        println!("Saved player entity for player {}", player.username);

        player
    }

    pub fn new_window_id(&mut self, window_type: WindowType) -> i32 {
        self.window_id += 1;
        self.current_window = Some(window_type);

        self.window_id
    }

    pub async fn send_packet(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let tx = crate::server::conn::PLAYER_SOCKET_MAP
            .read()
            .await
            .get(&self.uuid)
            .unwrap()
            .clone();

        let _ = tx.send(packet);

        Ok(())
    }

    pub async fn send_system_message(&self, message: String) -> anyhow::Result<()> {
        let tx = crate::server::conn::PLAYER_SOCKET_MAP
            .read()
            .await
            .get(&self.uuid)
            .unwrap()
            .clone();

        let mut buffer = Vec::new();
        send_system_chat_message(&mut buffer, message).await?;

        let _ = tx.send(buffer);

        Ok(())
    }

    pub async fn sync_player_inventory(&mut self) -> anyhow::Result<()> {
        let tx = crate::server::conn::PLAYER_SOCKET_MAP
            .read()
            .await
            .get(&self.uuid)
            .unwrap()
            .clone();

        let mut buffer = Vec::new();
        send_container_set_content(&mut buffer, 0, &self.inventory, self.carried_item).await?;
        let _ = tx.send(buffer);

        Ok(())
    }

    pub async fn sync_player_health(&mut self) -> anyhow::Result<()> {
        let tx = crate::server::conn::PLAYER_SOCKET_MAP
            .read()
            .await
            .get(&self.uuid)
            .unwrap()
            .clone();

        let mut buffer = Vec::new();
        send_set_health(&mut buffer, self).await?;
        let _ = tx.send(buffer);

        Ok(())
    }

    pub async fn sync_player_position(&mut self) -> anyhow::Result<()> {
        let tx = crate::server::conn::PLAYER_SOCKET_MAP
            .read()
            .await
            .get(&self.uuid)
            .unwrap()
            .clone();

        let mut buffer = Vec::new();
        send_sync_player_position(&mut buffer, self).await?;
        let _ = tx.send(buffer);

        self.fall_distance = 0.0;
        self.server_vy = 0.0;
        self.ignore_fall_for_ticks = 100;

        Ok(())
    }

    pub async fn set_inventory_slot(&mut self, slot: i16) -> anyhow::Result<()> {
        let tx = crate::server::conn::PLAYER_SOCKET_MAP
            .read()
            .await
            .get(&self.uuid)
            .unwrap()
            .clone();

        let mut buffer = Vec::new();
        send_container_set_slot(&mut buffer, 0, slot, self.inventory[slot as usize]).await?;
        let _ = tx.send(buffer);

        Ok(())
    }

    pub async fn give_item(&mut self, item_id: i32, count: i32) -> anyhow::Result<()> {
        let mut remaining = count;

        for slot in 36..45 {
            if remaining <= 0 {
                break;
            }

            if let Some(item_stack) = &mut self.inventory[slot] {
                if item_stack.item_id == item_id {
                    let space = 64 - item_stack.count as i32;
                    let to_add = space.min(remaining);
                    item_stack.count += to_add as u8;
                    remaining -= to_add;
                }
            } else {
                let to_add = 64.min(remaining);
                self.inventory[slot] = Some(super::items::ItemStack {
                    item_id,
                    count: to_add as u8,
                });
                remaining -= to_add;
            }
        }

        if remaining > 0 {
            for slot in 9..36 {
                if remaining <= 0 {
                    break;
                }

                if let Some(item_stack) = &mut self.inventory[slot] {
                    if item_stack.item_id == item_id {
                        let space = 64 - item_stack.count as i32;
                        let to_add = space.min(remaining);
                        item_stack.count += to_add as u8;
                        remaining -= to_add;
                    }
                } else {
                    let to_add = 64.min(remaining);
                    self.inventory[slot] = Some(super::items::ItemStack {
                        item_id,
                        count: to_add as u8,
                    });
                    remaining -= to_add;
                }
            }
        }

        self.sync_player_inventory().await?;

        Ok(())
    }

    pub async fn set_gamemode(&mut self, gamemode: Gamemode) -> anyhow::Result<()> {
        self.gamemode = gamemode;

        let tx = crate::server::conn::PLAYER_SOCKET_MAP
            .read()
            .await
            .get(&self.uuid)
            .unwrap()
            .clone();

        let mut buffer = Vec::new();
        send_game_event(
            &mut buffer,
            GameEvent::ChangeGameMode as u8,
            gamemode as u8 as f32,
        )
        .await?;
        let _ = tx.send(buffer);

        let all_tx = crate::server::conn::PLAYER_SOCKET_MAP
            .read()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>();

        let mut buffer = Vec::new();
        send_player_info_update(
            &mut buffer,
            vec![self],
            vec![PlayerAction::UpdateGameMode(gamemode as i32)],
        )
        .await?;

        for tx in all_tx {
            let _ = tx.send(buffer.clone());
        }

        Ok(())
    }

    pub async fn damage(
        &mut self,
        amount: i32,
        source_type_id: i32,
        source_cause_id: i32,
        source_direct_id: i32,
        skip_exhaustion: bool,
    ) -> anyhow::Result<()> {
        self.health -= amount as f32;
        let tx = crate::server::conn::PLAYER_SOCKET_MAP
            .read()
            .await
            .get(&self.uuid)
            .unwrap()
            .clone();

        let mut buffer = Vec::new();
        send_damage_event(
            &mut buffer,
            self.id,
            source_type_id,
            source_cause_id,
            source_direct_id,
        )
        .await?;
        let _ = tx.send(buffer);

        if !skip_exhaustion {
            self.add_exhaustion(Player::EX_DAMAGE_TAKEN).await;
        }

        self.stats_changed = true;

        if self.health <= 0.0 {
            self.health = 0.0;
            self.dead = true;

            for i in 0..45 {
                if let Some(item) = self.inventory[i].take()
                    && source_type_id != 32
                {
                    crate::types::entity::spawn_item_drop(
                        item,
                        Some(self.uuid.clone()),
                        self.x + (rand::random::<f64>() - 0.5) * 2.0,
                        self.y + 1.0,
                        self.z + (rand::random::<f64>() - 0.5) * 2.0,
                    );
                }
            }
        }

        Ok(())
    }

    pub async fn respawn(&mut self) -> anyhow::Result<()> {
        let tx = crate::server::conn::PLAYER_SOCKET_MAP
            .read()
            .await
            .get(&self.uuid)
            .unwrap()
            .clone();

        let mut buffer = Vec::new();
        send_respawn(&mut buffer, self).await?;
        let _ = tx.send(buffer);

        self.health = 20.0;
        self.hunger = 20;
        self.saturation = 5.0;

        let region_arc = get_region(0, 0).await;
        let chunk = get_chunk(&region_arc, 0, 0).await;
        let surface_y = chunk.get_surface_y(0, 0) as f64;

        self.x = 0.0;
        self.y = surface_y + 1.0;
        self.z = 0.0;

        self.vx = 0.0;
        self.vy = 0.0;
        self.vz = 0.0;
        self.server_vy = 0.0;

        self.yaw = 0.0;
        self.pitch = 0.0;

        self.dead = false;
        self.chunks_loaded = false;

        self.sync_player_health().await?;
        self.sync_player_position().await?;

        Ok(())
    }

    pub async fn send_hand_swing(&mut self, main_hand: bool) -> anyhow::Result<()> {
        let entity = crate::types::entity::ENTITIES
            .read()
            .await
            .get(&self.id)
            .cloned()
            .unwrap();

        entity.send_hand_swing(main_hand).await?;

        Ok(())
    }

    pub async fn add_exhaustion(&mut self, amount: f32) {
        if self.gamemode != Gamemode::Survival {
            return;
        }

        self.exhaustion += amount;

        while self.exhaustion >= 4.0 {
            self.exhaustion -= 4.0;

            if self.saturation > 0.0 {
                self.saturation = (self.saturation - 1.0).max(0.0);
            } else {
                self.hunger = (self.hunger - 1).max(0);
            }

            self.stats_changed = true;
        }
    }

    pub async fn tick(&mut self) -> anyhow::Result<()> {
        if self.dead || !self.initial_sync_done {
            return Ok(());
        }

        if self.gamemode == Gamemode::Survival {
            self.regen_timer = (self.regen_timer + 1).min(80);
            self.starvation_timer = (self.starvation_timer + 1).min(80);

            if self.hunger < 6 {
                self.movement.sprinting = false;
            }

            if self.hunger >= 18 && self.health < 20.0 && self.regen_timer >= 80 {
                self.regen_timer = 0;
                self.health = (self.health + 1.0).min(20.0);
                self.add_exhaustion(Player::EX_HEART_REGEN).await;
                self.stats_changed = true;
            }

            if self.hunger == 0 && self.starvation_timer >= 80 {
                self.starvation_timer = 0;
                self.damage(1, 40, 0, 0, true).await?;
            }
        }

        if self.eating_ticks_left > 0 {
            self.eating_ticks_left -= 1;

            if self.eating_ticks_left == 0 {
                if let Some(item) = self.inventory[self.eating_slot as usize + 36].take() {
                    if let Some(food_data) = get_food_data(item.item_id) {
                        self.hunger = (self.hunger + food_data.0).min(20);
                        self.saturation = (self.saturation + food_data.1).min(self.hunger as f32);
                        self.stats_changed = true;

                        let tx = crate::server::conn::PLAYER_SOCKET_MAP
                            .read()
                            .await
                            .get(&self.uuid)
                            .unwrap()
                            .clone();

                        let mut buffer = Vec::new();
                        send_entity_event(&mut buffer, self.id, 9).await?;

                        let _ = tx.send(buffer);

                        if item.count > 1 {
                            self.inventory[self.eating_slot as usize + 36] =
                                Some(crate::types::items::ItemStack {
                                    item_id: item.item_id,
                                    count: item.count - 1,
                                });
                        }

                        self.sync_player_inventory().await?;
                    }
                }
            }
        }

        let mut move_x = 0.0;
        let mut move_z = 0.0;

        if self.movement.forward {
            move_z += 1.0;
        }

        if self.movement.backward {
            move_z -= 1.0;
        }

        if self.movement.left {
            move_x += 1.0;
        }

        if self.movement.right {
            move_x -= 1.0;
        }

        if move_x != 0.0 || move_z != 0.0 {
            let length = ((move_x * move_x + move_z * move_z) as f64).sqrt();
            move_x /= length;
            move_z /= length;

            let speed = if self.movement.sprinting { 0.28 } else { 0.216 };

            if self.movement.sneaking {
                move_x *= 0.3;
                move_z *= 0.3;
            }

            if self.movement.sprinting && !self.movement.sneaking {
                self.add_exhaustion(Player::EX_SPRINT * speed as f32).await;
            }

            let yaw_rad = (self.yaw as f64).to_radians();

            self.vx = move_x * yaw_rad.cos() - move_z * yaw_rad.sin();
            self.vz = move_x * yaw_rad.sin() + move_z * yaw_rad.cos();

            self.vx *= speed;
            self.vz *= speed;
        } else {
            self.vx = 0.0;
            self.vz = 0.0;
        }

        self.vy -= 0.08;

        if self.on_ground || self.flying {
            self.vy = 0.0;
        }

        self.x += self.vx;
        self.y += self.vy;
        self.z += self.vz;

        if !self.on_ground && !self.flying {
            if self.movement.jumping && !self.jump_applied {
                self.server_vy = 0.42;
                self.jump_applied = true;

                if self.movement.sprinting {
                    self.add_exhaustion(Player::EX_SPRINT_JUMP).await;
                } else {
                    self.add_exhaustion(Player::EX_JUMP).await;
                }
            }

            self.server_vy -= 0.08;

            if self.server_vy < 0.0 {
                self.fall_distance += -self.server_vy;
            } else {
                self.fall_distance = 0.0;
            }
        } else if self.on_ground {
            if self.gamemode == Gamemode::Survival
                && self.ignore_fall_for_ticks <= 0
                && self.fall_distance > 3.0
            {
                let damage = (self.fall_distance - 3.0).ceil() as i32;
                self.damage(damage, 10, 0, 0, false).await?;
            }

            self.fall_distance = 0.0;
            self.server_vy = 0.0;
            self.jump_applied = false;
        }

        // void damage
        if self.y < -64.0 && self.gamemode == Gamemode::Survival && self.ignore_fall_for_ticks <= 0
        {
            self.damage(2, 32, 0, 0, false).await?;
        }

        self.ignore_fall_for_ticks = self.ignore_fall_for_ticks.saturating_sub(1);

        if self.ignore_fall_for_ticks > 0 {
            self.fall_distance = 0.0;
            self.server_vy = 0.0;
        }

        if self.stats_changed {
            self.sync_player_health().await?;
            self.stats_changed = false;
        }

        if !self.chunks_loaded {
            return Ok(());
        }

        let last_chunk_area_center_x = (self.last_x as i32) >> 4;
        let last_chunk_area_center_z = (self.last_z as i32) >> 4;

        let current_chunk_area_center_x = (self.x as i32) >> 4;
        let current_chunk_area_center_z = (self.z as i32) >> 4;

        if last_chunk_area_center_x != current_chunk_area_center_x
            || last_chunk_area_center_z != current_chunk_area_center_z
        {
            let tx = if let Some(player_tx) = crate::server::conn::PLAYER_SOCKET_MAP
                .read()
                .await
                .get(&self.uuid)
            {
                player_tx.clone()
            } else {
                return Ok(());
            };

            let mut buffer = Vec::new();

            send_set_center_chunk(
                &mut buffer,
                current_chunk_area_center_x,
                current_chunk_area_center_z,
            )
            .await?;

            let _ = tx.send(buffer);

            let view_distance = crate::config::SERVER_CONFIG.view_distance as i32;
            let chunk_loading_width = view_distance * 2 + 7;
            let radius = chunk_loading_width / 2;

            let mut old_chunks = std::collections::HashSet::new();
            for cx in (last_chunk_area_center_x - radius)..=(last_chunk_area_center_x + radius) {
                for cz in (last_chunk_area_center_z - radius)..=(last_chunk_area_center_z + radius)
                {
                    old_chunks.insert((cx, cz));
                }
            }

            let mut chunks_to_send = Vec::new();
            for cx in
                (current_chunk_area_center_x - radius)..=(current_chunk_area_center_x + radius)
            {
                for cz in
                    (current_chunk_area_center_z - radius)..=(current_chunk_area_center_z + radius)
                {
                    if !old_chunks.contains(&(cx, cz)) {
                        chunks_to_send.push((cx, cz));
                    }
                }
            }

            chunks_to_send.sort_by_key(|(cx, cz)| {
                let dx = cx - current_chunk_area_center_x;
                let dz = cz - current_chunk_area_center_z;
                dx * dx + dz * dz
            });

            let username_clone = self.username.clone();

            tokio::spawn(async move {
                for (cx, cz) in chunks_to_send {
                    let region_arc = get_region(cx >> 5, cz >> 5).await;
                    let chunk = get_chunk(&region_arc, cx, cz).await;

                    let mut buffer = Vec::new();
                    let result = send_level_chunk_with_light(&mut buffer, &chunk).await;
                    let _ = tx.send(buffer);

                    if result.is_ok() {
                        crate::log::log(
                            fancy_log::LogLevel::Debug,
                            &format!("Sent chunk {}, {} to player {}", cx, cz, username_clone),
                        );
                    } else {
                        crate::log::log(
                            fancy_log::LogLevel::Warn,
                            &format!(
                                "Failed to send chunk {}, {} to player {}: {:?}",
                                cx,
                                cz,
                                username_clone,
                                result.err().unwrap()
                            ),
                        );
                    }
                }
            });

            self.last_x = self.x;
            self.last_z = self.z;
        }

        let all_entities = crate::types::entity::ENTITIES.read().await;
        let tx = if let Some(tx) = crate::server::conn::PLAYER_SOCKET_MAP
            .read()
            .await
            .get(&self.uuid)
        {
            tx.clone()
        } else {
            return Ok(());
        };

        let mut to_remove = Vec::new();

        for (id, entity) in all_entities.iter() {
            if id == &self.id {
                continue;
            }

            if !self.loaded_entities.contains(id)
                && (entity.x - self.x).abs() < 64.0
                && (entity.z - self.z).abs() < 64.0
            {
                let distance_squared = (entity.x - self.x).powi(2)
                    + (entity.y - self.y).powi(2)
                    + (entity.z - self.z).powi(2);

                if distance_squared < 64.0 * 64.0 && !self.loaded_entities.contains(id) {
                    let mut buffer = Vec::new();
                    send_spawn_entity(&mut buffer, entity).await.unwrap();
                    let _ = tx.send(buffer);

                    let mut buffer = Vec::new();
                    sent_set_entity_data(&mut buffer, entity).await.unwrap();
                    let _ = tx.send(buffer);

                    self.loaded_entities.push(*id);
                } else if distance_squared >= 64.0 * 64.0 && self.loaded_entities.contains(id) {
                    to_remove.push(*id);
                }
            }
        }

        let all_entity_ids = all_entities.keys().cloned().collect::<Vec<_>>();
        to_remove.extend(
            self.loaded_entities
                .iter()
                .filter(|id| !all_entity_ids.contains(id))
                .cloned(),
        );
        drop(all_entities);

        if !to_remove.is_empty() {
            let mut buffer = Vec::new();
            send_remove_entities(&mut buffer, to_remove.clone())
                .await
                .unwrap();
            let _ = tx.send(buffer);

            self.loaded_entities.retain(|id| !to_remove.contains(id));
        }

        let mut player_positions = PLAYER_POSITIONS.write().await;
        player_positions.insert(self.uuid.clone(), (self.x, self.y, self.z));

        let mut entities_write = crate::types::entity::ENTITIES.write().await;

        if let Some(entity) = entities_write.get_mut(&self.id) {
            entity.x = self.x;
            entity.y = self.y;
            entity.z = self.z;

            entity.vx = self.vx;
            entity.vy = self.vy;
            entity.vz = self.vz;

            entity.yaw = self.yaw;
            entity.pitch = self.pitch;

            entity.on_ground = self.on_ground;
        }

        self.ticks_since_time_update += 1;

        if self.ticks_since_time_update >= 20 {
            self.ticks_since_time_update = 0;

            let mut buffer = Vec::new();
            send_update_time(
                &mut buffer,
                crate::server::tick::TIMESTAMP.load(Ordering::Relaxed) as i64,
            )
            .await?;

            self.send_packet(buffer).await?;
        }

        Ok(())
    }
}
