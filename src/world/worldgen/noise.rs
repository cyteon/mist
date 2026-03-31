use fastnoise_lite::{FastNoiseLite, NoiseType, FractalType};
use once_cell::sync::Lazy;

pub static TERRAIN_NOISE: Lazy<FastNoiseLite> = Lazy::new(|| {
    let mut noise = FastNoiseLite::new();
    noise.set_seed(Some(crate::config::SERVER_CONFIG.world_seed as i32));
    noise.set_noise_type(Some(NoiseType::Perlin));
    noise.set_fractal_type(Some(FractalType::FBm));
    noise.set_fractal_octaves(Some(4));
    noise.set_fractal_lacunarity(Some(2.0));
    noise.set_fractal_gain(Some(0.5));
    noise.set_frequency(Some(1.0 / 512.0));
    noise
});

pub static CAVE_NOISE: Lazy<FastNoiseLite> = Lazy::new(|| {
    let mut noise = FastNoiseLite::new();
    noise.set_seed(Some(crate::config::SERVER_CONFIG.world_seed as i32 + 1));
    noise.set_noise_type(Some(NoiseType::Perlin));
    noise.set_fractal_type(Some(FractalType::FBm));
    noise.set_fractal_octaves(Some(3));
    noise.set_frequency(Some(1.0 / 64.0));
    noise
});

pub fn get_height(x: f32, z: f32) -> i32 {
    let value = TERRAIN_NOISE.get_noise_2d(x, z);
    (value * 40.0 + 72.0) as i32
}

pub fn is_cave(x: f32, y: f32, z: f32) -> bool {
    CAVE_NOISE.get_noise_3d(x, y, z) > 0.5
}