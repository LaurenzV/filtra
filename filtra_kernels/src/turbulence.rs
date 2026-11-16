use fearless_simd::{Level, Simd, dispatch, f32x4, f32x8, prelude::*};

use crate::simd::{pack, premultiply};
use crate::{IntRect, Target};

/// Perlin turbulence (`feTurbulence`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Turbulence {
    /// The base frequencies along the x- and y-axis, in cycles per pixel.
    pub base_frequency: [f32; 2],
    /// The number of octaves.
    pub octaves: u32,
    /// The seed of the random numbers.
    pub seed: i64,
    /// The pixels the noise tiles, if it is stitched.
    pub stitch_tile: Option<IntRect>,
    /// Whether to sum the octaves (`fractalNoise`) instead of their absolute values.
    pub fractal_noise: bool,
    /// The position of the noise's origin in pixels.
    pub origin: [f32; 2],
}

/// Fills the target with the noise, sampled at the top-left corner of each pixel.
pub fn turbulence(dst: Target<'_>, turbulence: &Turbulence) {
    let lattice = Lattice::new(turbulence.seed);
    let noise = Noise::new(turbulence);

    dispatch!(Level::new(), simd => render(simd, &lattice, &noise, dst));
}

// The spec allows clamping the number of octaves (§9.21); later octaves change the result by
// less than 2⁻²⁴.
const MAX_OCTAVES: u32 = 24;

const B_SIZE: usize = 0x100;
const PERLIN_N: f64 = 4096.0;

const RAND_M: i64 = 2147483647;
const RAND_A: i64 = 16807;
const RAND_Q: i64 = 127773;
const RAND_R: i64 = 2836;

#[inline(always)]
fn render<S: Simd>(simd: S, lattice: &Lattice, noise: &Noise, mut dst: Target<'_>) {
    let rect = dst.rect();
    let select = |b: u8| lattice.selector[usize::from(b)];
    let points: Vec<f64> = (rect.x0..rect.x1)
        .map(|x| (f64::from(x) - noise.origin[0]) * noise.frequency[0])
        .collect();
    let mut sums = vec![0.0f32; points.len().div_ceil(8) * 32];

    for (y, row) in dst.rows_mut() {
        sums.fill(0.0);

        let point_y = (f64::from(y) - noise.origin[1]) * noise.frequency[1];
        let mut factor = 1.0;

        for stitch in &noise.stitches {
            let (mut by0, ry0) = lattice_cell(point_y * factor + PERLIN_N);
            let mut by1 = by0.wrapping_add(1);

            if let Some(stitch) = stitch {
                by0 = wrap(by0, stitch.wrap_y, stitch.height);
                by1 = wrap(by1, stitch.wrap_y, stitch.height);
            }

            let (by0, by1) = (by0 as u8, by1 as u8);
            let sy = s_curve(ry0);
            let (ry0, ry1) = (f32x4::splat(simd, ry0), f32x4::splat(simd, ry0 - 1.0));
            let weight = (1.0 / factor) as f32;

            for (point_x, sum) in points.iter().zip(sums.as_chunks_mut::<4>().0) {
                let (mut bx0, rx0) = lattice_cell(point_x * factor + PERLIN_N);
                let mut bx1 = bx0.wrapping_add(1);

                if let Some(stitch) = stitch {
                    bx0 = wrap(bx0, stitch.wrap_x, stitch.width);
                    bx1 = wrap(bx1, stitch.wrap_x, stitch.width);
                }

                let (i, j) = (select(bx0 as u8), select(bx1 as u8));
                let corner = |i: u8, b: u8| {
                    f32x8::from_slice(
                        simd,
                        &lattice.gradients[usize::from(select(i.wrapping_add(b)))],
                    )
                };
                let sx = s_curve(rx0);
                let (rx0, rx1) = (f32x4::splat(simd, rx0), f32x4::splat(simd, rx0 - 1.0));
                let d00 = corner(i, by0) * rx0.combine(ry0);
                let d10 = corner(j, by0) * rx1.combine(ry0);
                let d01 = corner(i, by1) * rx0.combine(ry1);
                let d11 = corner(j, by1) * rx1.combine(ry1);
                let a = d00 + (d10 - d00) * sx;
                let b = d01 + (d11 - d01) * sx;
                let (x, y) = (a + (b - a) * sy).split();
                let value = x + y;
                let value = if noise.fractal { value } else { value.abs() };

                value
                    .mul_add(weight, f32x4::from_slice(simd, sum))
                    .store_slice(sum);
            }

            factor *= 2.0;
        }

        for (chunk, pixels) in row.chunks_mut(32).enumerate() {
            let sums = &sums[chunk * 32..][..32];
            let v: [f32x8<S>; 4] =
                core::array::from_fn(|k| f32x8::from_slice(simd, &sums[k * 8..][..8]));

            let (even0, odd0) = (v[0].unzip_low(v[1]), v[0].unzip_high(v[1]));
            let (even1, odd1) = (v[2].unzip_low(v[3]), v[2].unzip_high(v[3]));
            let channels = [
                even0.unzip_low(even1),
                odd0.unzip_low(odd1),
                even0.unzip_high(even1),
                odd0.unzip_high(odd1),
            ]
            .map(|c| if noise.fractal { (c + 1.0) * 0.5 } else { c });

            let mut bytes = [0; 32];
            pack(premultiply(channels), &mut bytes);
            pixels.copy_from_slice(&bytes[..pixels.len()]);
        }
    }
}

struct Lattice {
    selector: [u8; B_SIZE],
    // Each entry holds the x-components of the four channels' gradients, then the y-components,
    // so one load serves all channels and the two halves add up to the dot products.
    gradients: [[f32; 8]; B_SIZE],
}

impl Lattice {
    fn new(seed: i64) -> Self {
        let mut seed = setup_seed(seed);
        let mut next = || {
            seed = random(seed);
            seed
        };
        let mut component =
            || ((next() % (2 * B_SIZE as i64)) - B_SIZE as i64) as f64 / B_SIZE as f64;
        let mut gradients = [[0.0; 8]; B_SIZE];

        // The setup of SVG 1.1 (§15.24), which all implementations kept: Filter Effects 1 redraws
        // vectors that are zero or longer than 1, which gives a different pattern for a seed.
        for channel in 0..4 {
            for gradient in &mut gradients {
                let [x, y] = [component(), component()];
                let length = (x * x + y * y).sqrt();

                // The reference code divides a zero vector by its zero length, which gives NaN; it
                // stays zero instead.
                let [x, y] = if length > 0.0 {
                    [x / length, y / length]
                } else {
                    [x, y]
                };
                gradient[channel] = x as f32;
                gradient[channel + 4] = y as f32;
            }
        }

        let mut selector = core::array::from_fn(|i| i as u8);

        for i in (1..B_SIZE).rev() {
            let j = (next() % B_SIZE as i64) as usize;
            selector.swap(i, j);
        }

        Self {
            selector,
            gradients,
        }
    }
}

fn setup_seed(seed: i64) -> i64 {
    let seed = if seed <= 0 {
        -(seed % (RAND_M - 1)) + 1
    } else {
        seed
    };

    seed.min(RAND_M - 1)
}

fn random(seed: i64) -> i64 {
    let result = RAND_A * (seed % RAND_Q) - RAND_R * (seed / RAND_Q);

    if result <= 0 { result + RAND_M } else { result }
}

struct Noise {
    frequency: [f64; 2],
    stitches: Vec<Option<Stitch>>,
    fractal: bool,
    origin: [f64; 2],
}

#[derive(Clone, Copy)]
struct Stitch {
    width: i64,
    height: i64,
    wrap_x: i64,
    wrap_y: i64,
}

impl Noise {
    fn new(turbulence: &Turbulence) -> Self {
        let origin = turbulence.origin.map(f64::from);
        let mut frequency = turbulence.base_frequency.map(f64::from);
        let octaves = turbulence.octaves.min(MAX_OCTAVES) as usize;
        let mut stitches = vec![None; octaves];

        if let Some(tile) = turbulence.stitch_tile {
            let x = f64::from(tile.x0) - origin[0];
            let y = f64::from(tile.y0) - origin[1];
            let width = f64::from(tile.width());
            let height = f64::from(tile.height());

            frequency = [
                stitch_frequency(frequency[0], width),
                stitch_frequency(frequency[1], height),
            ];

            let width = (width * frequency[0] + 0.5) as i64;
            let height = (height * frequency[1] + 0.5) as i64;
            let mut stitch = Stitch {
                width,
                height,
                wrap_x: (x * frequency[0] + PERLIN_N + width as f64) as i64,
                wrap_y: (y * frequency[1] + PERLIN_N + height as f64) as i64,
            };

            // The reference code's integers overflow for huge frequencies; these wrap instead.
            for octave in &mut stitches {
                *octave = Some(stitch);
                stitch = Stitch {
                    width: stitch.width.wrapping_mul(2),
                    height: stitch.height.wrapping_mul(2),
                    wrap_x: stitch.wrap_x.wrapping_mul(2).wrapping_sub(PERLIN_N as i64),
                    wrap_y: stitch.wrap_y.wrapping_mul(2).wrapping_sub(PERLIN_N as i64),
                };
            }
        }

        Self {
            frequency,
            stitches,
            fractal: turbulence.fractal_noise,
            origin,
        }
    }
}

fn stitch_frequency(frequency: f64, size: f64) -> f64 {
    if frequency == 0.0 {
        return frequency;
    }

    let low = (size * frequency).floor() / size;
    let high = (size * frequency).ceil() / size;

    if frequency / low < high / frequency {
        low
    } else {
        high
    }
}

fn lattice_cell(t: f64) -> (i64, f32) {
    let cell = t.trunc();
    let offset = if t.is_finite() { t - cell } else { 0.0 };

    (cell as i64, offset as f32)
}

fn s_curve(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn wrap(b: i64, wrap: i64, size: i64) -> i64 {
    if b >= wrap { b.wrapping_sub(size) } else { b }
}
