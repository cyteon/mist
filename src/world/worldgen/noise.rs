pub use noise::{NoiseFn, Perlin};
use once_cell::sync::Lazy;

fn seeded(offset: u32) -> Perlin {
    Perlin::new((crate::config::SERVER_CONFIG.world_seed as u32).wrapping_add(offset))
}

pub static DENSITY: Lazy<Perlin> = Lazy::new(|| seeded(3));
pub static ENTRANCES: Lazy<Perlin> = Lazy::new(|| seeded(4));
pub static SPAGHETTI_A: Lazy<Perlin> = Lazy::new(|| seeded(5));
pub static SPAGHETTI_B: Lazy<Perlin> = Lazy::new(|| seeded(6));
pub static CHEESE: Lazy<Perlin> = Lazy::new(|| seeded(7));

const CONTINENTS_CAL: f64 = 1.71;
const EROSION_CAL: f64 = 1.37;
const RIDGES_CAL: f64 = 1.45;
const JAGGED_CAL: f64 = 1.62;

pub struct Octaves {
    octs: Vec<(Perlin, f64, f64)>,
}

impl Octaves {
    fn new(seed_base: u32, first: i32, amps: &[f64], xz_scale: f64, cal: f64) -> Self {
        let n = amps.len() as i32;
        let lowest = 2f64.powi(n - 1) / (2f64.powi(n) - 1.0);
        let mut octs = vec![];
        let mut wsum = 0.0;

        for (i, &a) in amps.iter().enumerate() {
            if a == 0.0 {
                continue;
            }

            let w = a * lowest / 2f64.powi(i as i32);
            wsum += w;

            octs.push((
                seeded(seed_base + i as u32),
                xz_scale * 2f64.powi(first + i as i32),
                w,
            ));
        }

        for o in octs.iter_mut() {
            o.2 *= cal / wsum;
        }

        Octaves { octs }
    }

    pub fn sample(&self, x: f64, z: f64) -> f64 {
        self.octs
            .iter()
            .map(|(p, f, w)| p.get([x * f, z * f]) * w)
            .sum()
    }
}

pub static CONTINENTS: Lazy<Octaves> = Lazy::new(|| {
    Octaves::new(
        100,
        -9,
        &[1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0],
        0.25,
        CONTINENTS_CAL,
    )
});

pub static EROSION: Lazy<Octaves> =
    Lazy::new(|| Octaves::new(120, -9, &[1.0, 1.0, 0.0, 1.0, 1.0], 0.25, EROSION_CAL));

pub static RIDGES: Lazy<Octaves> =
    Lazy::new(|| Octaves::new(140, -7, &[1.0, 2.0, 1.0, 0.0, 0.0, 0.0], 0.25, RIDGES_CAL));

pub static JAGGED: Lazy<Octaves> =
    Lazy::new(|| Octaves::new(160, -16, &[1.0; 8], 1500.0, JAGGED_CAL));

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

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t.clamp(0.0, 1.0)
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

#[derive(Clone, Copy, Default)]
pub struct Shape {
    pub offset: f64,
    pub factor: f64,
    pub jaggedness: f64,
}

pub fn sample_shape(wx: f64, wz: f64) -> Shape {
    let c = CONTINENTS.sample(wx, wz).clamp(-1.5, 1.5) as f32;
    let e = EROSION.sample(wx, wz).clamp(-1.5, 1.5) as f32;
    let w = RIDGES.sample(wx, wz).clamp(-1.5, 1.5) as f32;

    let cv = [c, e, super::vanilla::peaks_valleys(w), w];

    Shape {
        offset: super::vanilla::OFFSET_ADD as f64
            + super::vanilla::eval(super::vanilla::OFFSET, &cv) as f64,
        factor: super::vanilla::eval(super::vanilla::FACTOR, &cv) as f64,
        jaggedness: super::vanilla::eval(super::vanilla::JAGGEDNESS, &cv) as f64,
    }
}
