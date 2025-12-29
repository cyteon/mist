#[derive(Clone)]
pub struct Player {
    pub uuid: String,
    pub name: String,
    
    pub shared_secret: Option<Vec<u8>>,
    pub skin_texture: Option<String>,

    pub x: f64,
    pub y: f64,
    pub z: f64,

    pub vx: f64,
    pub vy: f64,
    pub vz: f64,

    pub yaw: f32,
    pub pitch: f32,

    pub initial_sync_done: bool,
    pub chat_index: i32,
}

impl Player {
    pub fn new(uuid: String, name: String) -> Self {
        Player {
            uuid,
            name,

            shared_secret: None,
            skin_texture: None,

            x: 0.0,
            y: 60.0,
            z: 0.0,

            vx: 0.0,
            vy: 0.0,
            vz: 0.0,

            yaw: 0.0,
            pitch: 0.0,

            initial_sync_done: false,
            chat_index: -1,
        }
    }
}