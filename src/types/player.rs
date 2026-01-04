use crate::{net::packets::clientbound::{chunk_data_with_light::send_chunk_data_with_light, set_center_chunk::send_set_center_chunk}, world::worldgen::get_region};

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

#[derive(Clone)]
pub struct Player {
    pub uuid: String,
    pub username: String,
    
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
    pub chat_index: i32,
    pub chunks_loaded: bool,
}

impl Player {
    pub fn new(uuid: String, username: String) -> Self {
        Player {
            uuid,
            username,

            shared_secret: None,
            textures: None,
            texture_signature: None,

            x: 0.0,
            y: 60.0,
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
        }
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

        crate::log::log(fancy_log::LogLevel::Debug, &format!("Player in chunk area center: {}, {}", (self.x as i32) >> 4, (self.z as i32) >> 4));

        if !self.chunks_loaded {
            return Ok(());
        }

        let last_chunk_area_center_x = (self.last_x as i32) >> 4;
        let last_chunk_area_center_z = (self.last_z as i32) >> 4;

        let current_chunk_area_center_x = (self.x as i32) >> 4;
        let current_chunk_area_center_z = (self.z as i32) >> 4;

        if last_chunk_area_center_x != current_chunk_area_center_x || last_chunk_area_center_z != current_chunk_area_center_z {
            let socket = crate::server::conn::PLAYER_SOCKET_MAP.read().await.get(&self.uuid).unwrap().clone();

            send_set_center_chunk(
                &mut *socket.lock().await,
                current_chunk_area_center_x,
                current_chunk_area_center_z
            ).await?;

            crate::log::log(fancy_log::LogLevel::Debug, &format!("Player {} moved to new chunk area center: {}, {}", self.username, current_chunk_area_center_x, current_chunk_area_center_z));

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

            crate::log::log(fancy_log::LogLevel::Debug, &format!("Player {} needs {} new chunks", self.username, chunks_to_send.len()));

            for (cx, cz) in chunks_to_send {
                let region: crate::world::chunks::Region = get_region(cx >> 5, cz >> 5).await.lock().await.clone();
                let chunk = region.chunks.iter().find(|chunk| chunk.x == cx && chunk.z == cz).unwrap();

                let mut socket = socket.lock().await;
                send_chunk_data_with_light(&mut *socket, &chunk).await?;
            }

            self.last_x = self.x;
            self.last_z = self.z;
        }

        Ok(())
    }
}