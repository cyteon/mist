pub use noise::{Perlin, NoiseFn};
use once_cell::sync::Lazy;

pub static PERLIN: Lazy<Perlin> = Lazy::new(|| {
    Perlin::new(crate::config::SERVER_CONFIG.world_seed as u32)
});

pub struct fBmOptions {
    pub octaves: u32,
    pub freq: f64,
    pub amp: f64,
    pub lacunarity: f64,
    pub gain: f64,
}

pub fn fbm(
    x: f64, y: f64, z: f64,
    opts: fBmOptions
) -> f64 {
    let mut value = 0.0;
    let mut freq = opts.freq;
    let mut amp = opts.amp;
    
    for _ in 0..opts.octaves {
        value += PERLIN.get([x * freq, y * freq, z * freq]) * amp;
        freq *= opts.lacunarity;
        amp *= opts.gain;
    }

    value
}

pub fn get_height_map(cx: i32, cz: i32) -> [[i32; 16]; 16] {
    let wx = (cx << 4) as f64;
    let wz = (cz << 4) as f64;

    let c00 = continental_height(fbm(wx, 0.0, wz, fBmOptions {
        octaves: 4,
        freq: 1.0 / 2048.0,
        amp: 1.0,
        lacunarity: 2.0,
        gain: 0.5,
    }));

    let c10 = continental_height(fbm(wx + 15.0, 0.0, wz, fBmOptions {
        octaves: 4,
        freq: 1.0 / 2048.0,
        amp: 1.0,
        lacunarity: 2.0,
        gain: 0.5,
    }));

    let c01 = continental_height(fbm(wx, 0.0, wz + 15.0, fBmOptions {
        octaves: 4,
        freq: 1.0 / 2048.0,
        amp: 1.0,
        lacunarity: 2.0,
        gain: 0.5,
    }));

    let c11 = continental_height(fbm(wx + 15.0, 0.0, wz + 15.0, fBmOptions {
        octaves: 4,
        freq: 1.0 / 2048.0,
        amp: 1.0,
        lacunarity: 2.0,
        gain: 0.5,
    }));
    
    let e00 = fbm(wx, 0.0, wz, fBmOptions {
        octaves: 4,
        freq: 1.0 / 256.0,
        amp: 1.0,
        lacunarity: 2.0,
        gain: 0.5,
    });

    let e10 = fbm(wx + 15.0, 0.0, wz, fBmOptions {
        octaves: 4,
        freq: 1.0 / 256.0,
        amp: 1.0,
        lacunarity: 2.0,
        gain: 0.5,
    });

    let e01 = fbm(wx, 0.0, wz + 15.0, fBmOptions {
        octaves: 4,
        freq: 1.0 / 256.0,
        amp: 1.0,
        lacunarity: 2.0,
        gain: 0.5,
    });

    let e11 = fbm(wx + 15.0, 0.0, wz + 15.0, fBmOptions {
        octaves: 4,
        freq: 1.0 / 256.0,
        amp: 1.0,
        lacunarity: 2.0,
        gain: 0.5,
    });

    let mut map = [[0; 16]; 16];

    for x in 0..16 {
        for z in 0..16 {
            let tx = x as f64 / 15.0;
            let tz = z as f64 / 15.0;

            let base = bilerp(c00, c10, c01, c11, tx, tz);
            let peak = bilerp(e00, e10, e01, e11, tx, tz);

            let bx = wx + x as f64;
            let bz = wz + z as f64;

            let details = fbm(bx, 0.0, bz, fBmOptions {
                octaves: 6,
                freq: 1.0 / 256.0,
                amp: 1.0,
                lacunarity: 2.0,
                gain: 0.5,
            });

            map[x as usize][z as usize] = (base + details * 30.0 * peak) as i32;
        }
    }

    map
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
        lerp(72.0, 220.0, (c - 0.4) / 0.6)
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn bilerp(a: f64, b: f64, c: f64, d: f64, tx: f64, tz: f64) -> f64 {
    let u = lerp(a, b, tx);
    let v = lerp(c, d, tx);
    lerp(u, v, tz)
}