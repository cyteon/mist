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
enum EntityType {
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
    pub fn tick(&mut self) {
        // todo do smth here ig
    }
}