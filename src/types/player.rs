use crate::{
    net::packets::clientbound::{
        chunk_data_with_light::send_chunk_data_with_light,
        set_center_chunk::send_set_center_chunk,
        system_chat_message::send_system_chat_message,
        container_set_content::send_container_set_content,
        container_set_slot::send_container_set_slot,
        game_event::{send_game_event, GameEvent},
        player_info_update::{send_player_info_update, PlayerAction}
    },
    
    world::{get_region, get_chunk}
};

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


#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum Gamemode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl Default for Gamemode {
    fn default() -> Self {
        Gamemode::Survival
    }
}

#[derive(Clone)]
pub struct Player {
    pub id: i32,
    pub uuid: String,
    pub username: String,

    pub inventory: [Option<super::items::ItemStack>; 45],
    pub current_slot: i16,

    pub is_op: bool,
    pub gamemode: Gamemode,
    
    pub shared_secret: Option<Vec<u8>>,
    pub textures: Option<String>,
    pub texture_signature: Option<String>,

    pub x: f64,
    pub y: f64,
    pub z: f64,

    // used to determine what chunks to send
    pub last_x: f64,
    pub last_z: f64,

    pub vx: f64,
    pub vy: f64,
    pub vz: f64,

    pub yaw: f32,
    pub pitch: f32,

    pub movement: PlayerMovement,

    pub initial_sync_done: bool,
    pub chunks_loaded: bool,
    pub chat_index: i32,
}

impl Player {
    pub fn new(uuid: String, username: String) -> Self {
        let mut player = Player {
            id: super::entity::next_entity_id(),
            uuid,
            username: username.clone(),
            
            inventory: [None; 45],
            current_slot: 0,

            is_op: false,
            gamemode: match crate::config::SERVER_CONFIG.default_gamemode.as_str() {
                "survival" => Gamemode::Survival,
                "creative" => Gamemode::Creative,
                "adventure" => Gamemode::Adventure,
                "spectator" => Gamemode::Spectator,
                _ => {
                    crate::log::log(fancy_log::LogLevel::Warn, &format!("Invalid default gamemode in config: {}, defaulting to survival", crate::config::SERVER_CONFIG.default_gamemode));
                    Gamemode::Survival
                }
            },

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
        };

        let player_save = crate::server::save::load_player(&player.uuid);

        if let Some(player_save) = player_save {
            player.inventory = player_save.inventory.try_into().unwrap_or_else(|_| [None; 45]);

            player.is_op = player_save.is_op;
            player.gamemode = player_save.gamemode;

            player.x = player_save.x;
            player.y = player_save.y;
            player.z = player_save.z;

            player.vx = player_save.vx;
            player.vy = player_save.vy;
            player.vz = player_save.vz;

            player.yaw = player_save.yaw;
            player.pitch = player_save.pitch;

            crate::log::log(fancy_log::LogLevel::Info, &format!("Loaded save for player {}", username));
        } else {
            player.inventory[36] = Some(super::items::ItemStack { item_id: super::items::GRASS_BLOCK, count: 64 });
            player.inventory[37] = Some(super::items::ItemStack { item_id: super::items::DIRT, count: 64 });
            player.inventory[38] = Some(super::items::ItemStack { item_id: super::items::STONE, count: 64 });
        }

        player
    }

    pub async fn send_system_message(&self, message: String) -> anyhow::Result<()> {
        let tx = crate::server::conn::PLAYER_SOCKET_MAP.read().await.get(&self.uuid).unwrap().clone();

        let mut buffer = Vec::new();
        send_system_chat_message(&mut buffer, message).await?;

        let _ = tx.send(buffer);

        Ok(())
    }

    pub async fn sync_player_inventory(&mut self) -> anyhow::Result<()> {
        let tx = crate::server::conn::PLAYER_SOCKET_MAP.read().await.get(&self.uuid).unwrap().clone();

        let mut buffer = Vec::new();
        send_container_set_content(&mut buffer, 0, &self.inventory).await?;
        let _ = tx.send(buffer);

        Ok(())
    }

    pub async fn set_inventory_slot(&mut self, slot: i16) -> anyhow::Result<()> {
        let tx = crate::server::conn::PLAYER_SOCKET_MAP.read().await.get(&self.uuid).unwrap().clone();

        let mut buffer = Vec::new();
        send_container_set_slot(&mut buffer, 0, slot, self.inventory[slot as usize].clone()).await?;
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
                self.inventory[slot] = Some(super::items::ItemStack { item_id, count: to_add as u8 });
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
                    self.inventory[slot] = Some(super::items::ItemStack { item_id, count: to_add as u8 });
                    remaining -= to_add;
                }
            }
        }

        self.sync_player_inventory().await?;

        Ok(())
    }

    pub async fn set_gamemode(&mut self, gamemode: Gamemode) -> anyhow::Result<()> {
        self.gamemode = gamemode;

        let tx = crate::server::conn::PLAYER_SOCKET_MAP.read().await.get(&self.uuid).unwrap().clone();

        let mut buffer = Vec::new();
        send_game_event(&mut buffer, GameEvent::ChangeGameMode as u8, gamemode as u8 as f32).await?;
        let _ = tx.send(buffer);

        let all_tx = crate::server::conn::PLAYER_SOCKET_MAP.read().await.values().cloned().collect::<Vec<_>>();

        let mut buffer = Vec::new();
        send_player_info_update(
            &mut buffer,
            vec![self],
            vec![PlayerAction::UpdateGameMode(gamemode as i32)]
        ).await?;

        for tx in all_tx {
            let _ = tx.send(buffer.clone());
        }

        Ok(())
    }

    pub async fn tick(&mut self) -> anyhow::Result<()> {
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

            let yaw_rad = (self.yaw as f64).to_radians();

            self.vx = move_x * yaw_rad.cos() - move_z * yaw_rad.sin();
            self.vz = move_x * yaw_rad.sin() + move_z * yaw_rad.cos();

            self.vx *= speed;
            self.vz *= speed;
        } else {
            self.vx = 0.0;
            self.vz = 0.0;
        }

        self.x += self.vx;
        self.z += self.vz;

        if !self.chunks_loaded {
            return Ok(());
        }

        let last_chunk_area_center_x = (self.last_x as i32) >> 4;
        let last_chunk_area_center_z = (self.last_z as i32) >> 4;

        let current_chunk_area_center_x = (self.x as i32) >> 4;
        let current_chunk_area_center_z = (self.z as i32) >> 4;

        if last_chunk_area_center_x != current_chunk_area_center_x || last_chunk_area_center_z != current_chunk_area_center_z {
            let tx = if let Some(player_tx) = crate::server::conn::PLAYER_SOCKET_MAP.read().await.get(&self.uuid) {
                player_tx.clone()
            } else {
                return Ok(());
            };

            let mut buffer = Vec::new();

            send_set_center_chunk(
                &mut buffer,
                current_chunk_area_center_x,
                current_chunk_area_center_z
            ).await?;

            let _ = tx.send(buffer);

            let view_distance = crate::config::SERVER_CONFIG.view_distance as i32;
            let chunk_loading_width = view_distance * 2 + 7;
            let radius = chunk_loading_width / 2;

            let mut old_chunks = std::collections::HashSet::new();
            for cx in (last_chunk_area_center_x - radius)..=(last_chunk_area_center_x + radius) {
                for cz in (last_chunk_area_center_z - radius)..=(last_chunk_area_center_z + radius) {
                    old_chunks.insert((cx, cz));
                }
            }

            let mut chunks_to_send = Vec::new();
            for cx in (current_chunk_area_center_x - radius)..=(current_chunk_area_center_x + radius) {
                for cz in (current_chunk_area_center_z - radius)..=(current_chunk_area_center_z + radius) {
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
                    let result = send_chunk_data_with_light(&mut buffer, &chunk).await;
                    let _ = tx.send(buffer);

                    if result.is_ok() {
                        crate::log::log(fancy_log::LogLevel::Debug, &format!("Sent chunk {}, {} to player {}", cx, cz, username_clone));
                    } else {
                        crate::log::log(fancy_log::LogLevel::Warn, &format!("Failed to send chunk {}, {} to player {}: {:?}", cx, cz, username_clone, result.err().unwrap()));
                    }
                }
            });

            self.last_x = self.x;
            self.last_z = self.z;
        }

        Ok(())
    }
}