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

    pub vx: f64,
    pub vy: f64,
    pub vz: f64,

    pub yaw: f32,
    pub pitch: f32,

    pub movement: PlayerMovement,

    pub initial_sync_done: bool,
    pub chat_index: i32,
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
        }
    }

    pub async fn tick(&mut self) {
        let mut move_x = 0.0;
        let mut move_z = 0.0;

        dbg!(&self.movement.forward, &self.movement.backward, &self.movement.left, &self.movement.right);

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
    }
}