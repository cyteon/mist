use noise::{NoiseFn, Perlin};
use once_cell::sync::Lazy;

pub static PERLIN: Lazy<Perlin> = Lazy::new(|| {
    Perlin::new(crate::config::SERVER_CONFIG.world_seed as u32)
});

fn fbm(
    perlin: &Perlin,
    x: f64,
    z: f64,
    octaves: u32,
    freq: f64,
    amp: f64,
    lacunarity: f64,
    gain: f64,
) -> f64 {
    let mut value = 0.0;
    let mut freq = freq;
    let mut amp = amp;
    
    for _ in 0..octaves {
        value += perlin.get([x * freq, z * freq]) * amp;
        freq *= lacunarity;
        amp *= gain;
    }

    value
}

pub fn get_height(perlin: &Perlin, x: f64, z: f64) -> i32 {
    let continents = fbm(
        perlin, x, z,
        4,
        1.0 / 2048.0,
        1.0,
        2.0,
        0.5
    );

    let details = fbm(
        perlin, x, z,
        6,
        1.0 / 256.0,
        1.0,
        2.0,
        0.5
    );

    let combined = continents * 0.6 + details * 0.4;
    (combined * 40.0 + 72.0) as i32
}