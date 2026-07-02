pub use noise::{NoiseFn, Perlin};
use once_cell::sync::Lazy;

pub static PERLIN: Lazy<Perlin> =
    Lazy::new(|| Perlin::new(crate::config::SERVER_CONFIG.world_seed as u32));

fn seeded(offset: u32) -> Perlin {
    Perlin::new((crate::config::SERVER_CONFIG.world_seed as u32).wrapping_add(offset))
}

pub static CONTINENTS: Lazy<Perlin> = Lazy::new(|| seeded(0));
pub static EROSION: Lazy<Perlin> = Lazy::new(|| seeded(1));
pub static RIDGES: Lazy<Perlin> = Lazy::new(|| seeded(2));
pub static DENSITY: Lazy<Perlin> = Lazy::new(|| seeded(3));
pub static ENTRANCES: Lazy<Perlin> = Lazy::new(|| seeded(4));
pub static SPAGHETTI_A: Lazy<Perlin> = Lazy::new(|| seeded(5));
pub static SPAGHETTI_B: Lazy<Perlin> = Lazy::new(|| seeded(6));
pub static CHEESE: Lazy<Perlin> = Lazy::new(|| seeded(7));

pub struct fBmOptions {
    pub octaves: u32,
    pub freq: f64,
    pub amp: f64,
    pub lacunarity: f64,
    pub gain: f64,
}

pub fn fbm(x: f64, y: f64, z: f64, opts: fBmOptions) -> f64 {
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

pub fn fbm2(perlin: &Perlin, x: f64, z: f64, octaves: u32, freq: f64) -> f64 {
    let mut value = 0.0;
    let mut f = freq;
    let mut amp = 1.0;
    let mut norm = 0.0;

    for _ in 0..octaves {
        value += perlin.get([x * f, z * f]) * amp;
        norm += amp;
        f *= 2.0;
        amp *= 0.5;
    }

    value / norm
}

pub fn fbm3(perlin: &Perlin, x: f64, y: f64, z: f64, octaves: u32, freq: f64) -> f64 {
    let mut value = 0.0;
    let mut f = freq;
    let mut amp = 1.0;
    let mut norm = 0.0;

    for _ in 0..octaves {
        value += perlin.get([x * f, y * f, z * f]) * amp;
        norm += amp;
        f *= 2.0;
        amp *= 0.5;
    }

    value / norm
}

pub fn get_height_map(cx: i32, cz: i32) -> [[i32; 16]; 16] {
    let wx = (cx << 4) as f64;
    let wz = (cz << 4) as f64;

    let c00 = continental_height(fbm(
        wx,
        0.0,
        wz,
        fBmOptions {
            octaves: 4,
            freq: 1.0 / 2048.0,
            amp: 1.0,
            lacunarity: 2.0,
            gain: 0.5,
        },
    ));

    let c10 = continental_height(fbm(
        wx + 15.0,
        0.0,
        wz,
        fBmOptions {
            octaves: 4,
            freq: 1.0 / 2048.0,
            amp: 1.0,
            lacunarity: 2.0,
            gain: 0.5,
        },
    ));

    let c01 = continental_height(fbm(
        wx,
        0.0,
        wz + 15.0,
        fBmOptions {
            octaves: 4,
            freq: 1.0 / 2048.0,
            amp: 1.0,
            lacunarity: 2.0,
            gain: 0.5,
        },
    ));

    let c11 = continental_height(fbm(
        wx + 15.0,
        0.0,
        wz + 15.0,
        fBmOptions {
            octaves: 4,
            freq: 1.0 / 2048.0,
            amp: 1.0,
            lacunarity: 2.0,
            gain: 0.5,
        },
    ));

    let e00 = fbm(
        wx,
        0.0,
        wz,
        fBmOptions {
            octaves: 4,
            freq: 1.0 / 256.0,
            amp: 1.0,
            lacunarity: 2.0,
            gain: 0.5,
        },
    );

    let e10 = fbm(
        wx + 15.0,
        0.0,
        wz,
        fBmOptions {
            octaves: 4,
            freq: 1.0 / 256.0,
            amp: 1.0,
            lacunarity: 2.0,
            gain: 0.5,
        },
    );

    let e01 = fbm(
        wx,
        0.0,
        wz + 15.0,
        fBmOptions {
            octaves: 4,
            freq: 1.0 / 256.0,
            amp: 1.0,
            lacunarity: 2.0,
            gain: 0.5,
        },
    );

    let e11 = fbm(
        wx + 15.0,
        0.0,
        wz + 15.0,
        fBmOptions {
            octaves: 4,
            freq: 1.0 / 256.0,
            amp: 1.0,
            lacunarity: 2.0,
            gain: 0.5,
        },
    );

    let mut map = [[0; 16]; 16];

    for x in 0..16 {
        for z in 0..16 {
            let tx = x as f64 / 15.0;
            let tz = z as f64 / 15.0;

            let base = bilerp(c00, c10, c01, c11, tx, tz);
            let peak = bilerp(e00, e10, e01, e11, tx, tz);

            let bx = wx + x as f64;
            let bz = wz + z as f64;

            let details = fbm(
                bx,
                0.0,
                bz,
                fBmOptions {
                    octaves: 6,
                    freq: 1.0 / 256.0,
                    amp: 1.0,
                    lacunarity: 2.0,
                    gain: 0.5,
                },
            );

            map[x as usize][z as usize] = (base + details * 30.0 * peak) as i32;
        }
    }

    map
}

fn continental_height(c: f64) -> f64 {
    if c < -0.45 {
        32.0
    } else if c < -0.15 {
        lerp(32.0, 56.0, (c + 0.45) / 0.3)
    } else if c < -0.02 {
        lerp(56.0, 62.0, (c + 0.2) / 0.13)
    } else if c < 0.25 {
        lerp(64.0, 78.0, (c + 0.2) / 0.27)
    } else {
        lerp(78.0, 110.0, (c - 0.25) / 0.75)
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

pub fn trilerp(
    d000: f64,
    d100: f64,
    d001: f64,
    d101: f64,
    d010: f64,
    d110: f64,
    d011: f64,
    d111: f64,
    tx: f64,
    ty: f64,
    tz: f64,
) -> f64 {
    let x00 = lerp(d000, d100, tx);
    let x01 = lerp(d001, d101, tx);
    let x10 = lerp(d010, d110, tx);
    let x11 = lerp(d011, d111, tx);

    let z0 = lerp(x00, x01, tz);
    let z1 = lerp(x10, x11, tz);

    lerp(z0, z1, ty)
}

pub fn sample_shape(wx: f64, wz: f64) -> (f64, f64) {
    let c = fbm2(&CONTINENTS, wx, wz, 6, 1.0 / 2048.0);
    let e = fbm2(&EROSION, wx, wz, 4, 1.0 / 256.0);
    let w = fbm2(&RIDGES, wx, wz, 4, 1.0 / 512.0);

    let pv = -((w.abs() - 2.0 / 3.0).abs() - 1.0 / 3.0) * 3.0;

    let base = continental_height(c);

    let mountain = (-e).clamp(0.0, 1.0);
    let mountain = mountain * mountain;
    let inland = ((c + 0.05) / 0.3).clamp(0.0, 1.0);

    let offset = base + pv * inland * (6.0 + 80.0 * mountain);
    let factor = 3.0 + 22.0 * mountain * inland;

    (offset, factor)
}
