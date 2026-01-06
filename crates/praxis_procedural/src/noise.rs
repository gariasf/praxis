//! Noise function implementations for procedural texture generation.
//!
//! This module provides CPU-side implementations of various noise functions
//! used for procedural texture generation. These implementations match the
//! GPU compute shader versions for testing and preview purposes.

/// Generates 2D Perlin noise at the given coordinates.
///
/// Perlin noise is a gradient noise function that produces smooth, natural-looking patterns.
/// It's useful for terrain, clouds, wood grain, and other organic textures.
///
/// # Arguments
///
/// * `x` - X coordinate
/// * `y` - Y coordinate
/// * `seed` - Random seed for reproducible results
///
/// # Returns
///
/// A noise value in the range [-1, 1]
pub fn perlin_noise(x: f32, y: f32, seed: u32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let xf = x - x.floor();
    let yf = y - y.floor();

    let u = fade(xf);
    let v = fade(yf);

    let aa = grad_2d(hash_2d(xi, yi, seed), xf, yf);
    let ba = grad_2d(hash_2d(xi + 1, yi, seed), xf - 1.0, yf);
    let ab = grad_2d(hash_2d(xi, yi + 1, seed), xf, yf - 1.0);
    let bb = grad_2d(hash_2d(xi + 1, yi + 1, seed), xf - 1.0, yf - 1.0);

    let x1 = lerp(aa, ba, u);
    let x2 = lerp(ab, bb, u);
    lerp(x1, x2, v)
}

/// Generates 2D Simplex noise at the given coordinates.
///
/// Simplex noise is an improved version of Perlin noise with better visual isotropy
/// and computational efficiency. It has fewer directional artifacts.
///
/// # Arguments
///
/// * `x` - X coordinate
/// * `y` - Y coordinate
/// * `seed` - Random seed for reproducible results
///
/// # Returns
///
/// A noise value in the range [-1, 1]
pub fn simplex_noise(x: f32, y: f32, seed: u32) -> f32 {
    const F2: f32 = 0.366_025_4; // 0.5 * (sqrt(3.0) - 1.0)
    const G2: f32 = 0.211_324_87; // (3.0 - sqrt(3.0)) / 6.0

    let s = (x + y) * F2;
    let xs = x + s;
    let ys = y + s;
    let i = xs.floor();
    let j = ys.floor();

    let t = (i + j) * G2;
    let x0 = x - (i - t);
    let y0 = y - (j - t);

    let (i1, j1) = if x0 > y0 { (1.0, 0.0) } else { (0.0, 1.0) };

    let x1 = x0 - i1 + G2;
    let y1 = y0 - j1 + G2;
    let x2 = x0 - 1.0 + 2.0 * G2;
    let y2 = y0 - 1.0 + 2.0 * G2;

    let ii = i as i32;
    let jj = j as i32;

    let n0 = simplex_contrib(x0, y0, hash_2d(ii, jj, seed));
    let n1 = simplex_contrib(x1, y1, hash_2d(ii + i1 as i32, jj + j1 as i32, seed));
    let n2 = simplex_contrib(x2, y2, hash_2d(ii + 1, jj + 1, seed));

    40.0 * (n0 + n1 + n2)
}

/// Generates 2D Worley (cellular) noise at the given coordinates.
///
/// Worley noise is based on distance to feature points and creates cellular patterns.
/// It's useful for stone, cells, cracked earth, and other organic patterns.
///
/// # Arguments
///
/// * `x` - X coordinate
/// * `y` - Y coordinate
/// * `seed` - Random seed for reproducible results
/// * `cell_size` - Size of each cell (default: 1.0)
///
/// # Returns
///
/// A noise value in the range [0, 1] representing distance to nearest feature point
pub fn worley_noise(x: f32, y: f32, seed: u32, cell_size: f32) -> f32 {
    let x = x / cell_size;
    let y = y / cell_size;

    let cell_x = x.floor() as i32;
    let cell_y = y.floor() as i32;

    let mut min_dist = f32::MAX;

    for dy in -1..=1 {
        for dx in -1..=1 {
            let neighbor_x = cell_x + dx;
            let neighbor_y = cell_y + dy;

            let hash = hash_2d(neighbor_x, neighbor_y, seed);
            let point_x = neighbor_x as f32 + random_float(hash);
            let point_y = neighbor_y as f32 + random_float(hash.wrapping_mul(2654435761));

            let diff_x = x - point_x;
            let diff_y = y - point_y;
            let dist = (diff_x * diff_x + diff_y * diff_y).sqrt();

            min_dist = min_dist.min(dist);
        }
    }

    min_dist.min(1.0)
}

/// Generates fractal Brownian motion (fBm) noise by summing multiple octaves.
///
/// fBm creates natural-looking patterns by combining multiple frequencies of noise
/// with decreasing amplitude.
///
/// # Arguments
///
/// * `x` - X coordinate
/// * `y` - Y coordinate
/// * `seed` - Random seed
/// * `octaves` - Number of noise layers to combine
/// * `persistence` - How much each octave contributes (typically 0.5)
/// * `lacunarity` - Frequency multiplier between octaves (typically 2.0)
/// * `noise_fn` - The base noise function to use
///
/// # Returns
///
/// A noise value in an octave-dependent range
pub fn fbm_noise<F>(
    x: f32,
    y: f32,
    seed: u32,
    octaves: u32,
    persistence: f32,
    lacunarity: f32,
    noise_fn: F,
) -> f32
where
    F: Fn(f32, f32, u32) -> f32,
{
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for i in 0..octaves {
        value += noise_fn(x * frequency, y * frequency, seed.wrapping_add(i)) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    value / max_value
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

fn hash_2d(x: i32, y: i32, seed: u32) -> u32 {
    let mut h = seed;
    h = h.wrapping_mul(374761393).wrapping_add(x as u32);
    h = h.wrapping_mul(668265263).wrapping_add(y as u32);
    h ^= h >> 13;
    h = h.wrapping_mul(1274126177);
    h ^= h >> 16;
    h
}

fn grad_2d(hash: u32, x: f32, y: f32) -> f32 {
    let h = hash & 7;
    let u = if h < 4 { x } else { y };
    let v = if h < 4 { y } else { x };
    let u = if h & 1 == 0 { u } else { -u };
    let v = if h & 2 == 0 { v } else { -v };
    u + v
}

fn simplex_contrib(x: f32, y: f32, hash: u32) -> f32 {
    let t = 0.5 - x * x - y * y;
    if t < 0.0 {
        0.0
    } else {
        let t = t * t;
        t * t * grad_2d(hash, x, y)
    }
}

fn random_float(hash: u32) -> f32 {
    hash as f32 / u32::MAX as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perlin_noise_range() {
        for _ in 0..100 {
            let x = rand::random::<f32>() * 100.0;
            let y = rand::random::<f32>() * 100.0;
            let value = perlin_noise(x, y, 0);
            assert!((-1.0..=1.0).contains(&value));
        }
    }

    #[test]
    fn test_simplex_noise_range() {
        for _ in 0..100 {
            let x = rand::random::<f32>() * 100.0;
            let y = rand::random::<f32>() * 100.0;
            let value = simplex_noise(x, y, 0);
            assert!((-1.5..=1.5).contains(&value));
        }
    }

    #[test]
    fn test_worley_noise_range() {
        for _ in 0..100 {
            let x = rand::random::<f32>() * 100.0;
            let y = rand::random::<f32>() * 100.0;
            let value = worley_noise(x, y, 0, 1.0);
            assert!((0.0..=1.0).contains(&value));
        }
    }

    #[test]
    fn test_noise_determinism() {
        let x = 42.0;
        let y = 17.0;
        let seed = 12345;

        let p1 = perlin_noise(x, y, seed);
        let p2 = perlin_noise(x, y, seed);
        assert_eq!(p1, p2);

        let s1 = simplex_noise(x, y, seed);
        let s2 = simplex_noise(x, y, seed);
        assert_eq!(s1, s2);

        let w1 = worley_noise(x, y, seed, 1.0);
        let w2 = worley_noise(x, y, seed, 1.0);
        assert_eq!(w1, w2);
    }

    #[test]
    fn test_noise_variation_with_seed() {
        let x = 10.5;
        let y = 20.5;

        let p1 = perlin_noise(x, y, 0);
        let p2 = perlin_noise(x, y, 12345);
        assert_ne!(p1, p2, "Perlin noise should vary with different seeds");

        let s1 = simplex_noise(x, y, 0);
        let s2 = simplex_noise(x, y, 12345);
        assert_ne!(s1, s2, "Simplex noise should vary with different seeds");

        let w1 = worley_noise(x, y, 0, 1.0);
        let w2 = worley_noise(x, y, 12345, 1.0);
        assert_ne!(w1, w2, "Worley noise should vary with different seeds");
    }

    #[test]
    fn test_fbm_noise() {
        let value = fbm_noise(5.0, 5.0, 0, 4, 0.5, 2.0, perlin_noise);
        assert!((-1.0..=1.0).contains(&value));

        let value2 = fbm_noise(5.0, 5.0, 0, 1, 0.5, 2.0, perlin_noise);
        let single = perlin_noise(5.0, 5.0, 0);
        assert!((value2 - single).abs() < 0.001);
    }

    #[test]
    fn test_fade_function() {
        assert_eq!(fade(0.0), 0.0);
        assert_eq!(fade(1.0), 1.0);
        let mid = fade(0.5);
        assert!(mid > 0.0 && mid < 1.0);
    }

    #[test]
    fn test_hash_consistency() {
        let h1 = hash_2d(0, 0, 0);
        let h2 = hash_2d(0, 0, 0);
        assert_eq!(h1, h2);

        let h3 = hash_2d(1, 0, 0);
        assert_ne!(h1, h3);

        let h4 = hash_2d(0, 1, 0);
        assert_ne!(h1, h4);

        let h5 = hash_2d(0, 0, 1);
        assert_ne!(h1, h5);
    }
}
