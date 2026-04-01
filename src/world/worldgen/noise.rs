pub use noise::{Perlin, NoiseFn};
use once_cell::sync::Lazy;

pub static PERLIN: Lazy<Perlin> = Lazy::new(|| {
    Perlin::new(crate::config::SERVER_CONFIG.world_seed as u32)
});

fn fbm(
    x: f64, y: f64, z: f64,
    octaves: u32, freq: f64, amp: f64,
    lacunarity: f64, gain: f64,
) -> f64 {
    let mut value = 0.0;
    let mut freq = freq;
    let mut amp = amp;
    
    for _ in 0..octaves {
        value += PERLIN.get([x * freq, y * freq, z * freq]) * amp;
        freq *= lacunarity;
        amp *= gain;
    }

    value
}

pub fn get_height(x: f64, z: f64) -> i32 {
    let continents = fbm(
        x, 0.0, z,
        4, 1.0 / 2048.0, 
        1.0, 2.0, 0.5
    );

    let erosion = fbm(
        x, 0.0, z,
        4, 1.0 / 1024.0,
        1.0, 2.0, 0.5
    ).abs();

    let details = fbm(
        x, 0.0, z,
        6, 1.0 / 256.0,
        1.0, 2.0, 0.5
    );

    let base = continental_height(continents);
    let peak = ((erosion + 1.0) / 2.0).powi(2);

    (base + details * 30.0 * peak) as i32
}

fn continental_height(c: f64) -> f64 {
    if c < -0.4 {
        30.0
    } else if c < -0.2 {
        lerp(30.0, 58.0, (c + 0.4) / 0.2)
    } else if c < 0.0 {
        lerp(58.0, 63.0, (c + 0.2) / 0.2)
    } else if c < 0.4 {
        lerp(63.0, 72.0, c / 0.4)
    } else {
        lerp(72.0, 110.0, (c - 0.4) / 0.6)
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

pub fn is_cave(x: f64, y: f64, z: f64) -> bool {
    let value = fbm(
        x, y, z, 
        3, 1.0 / 64.0,
        1.0, 2.0, 0.5
    );

    if value > 0.6 {
        return true;
    }

    let v1 = PERLIN.get([x / 64.0, y / 64.0, z / 64.0]);
    let v2 = PERLIN.get([x / 64.0 + 100.0, y / 64.0 + 100.0, z / 64.0 + 100.0]);

    (v1 * v1 + v2 * v2) < 0.005
}