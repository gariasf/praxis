// Noise function implementations for GPU compute shaders
// These implementations match the CPU versions in noise.rs
//
// # Overview of Noise Functions
//
// This file provides three types of coherent noise suitable for procedural generation:
// 1. **Perlin Noise**: Gradient-based, smooth transitions
// 2. **Simplex Noise**: Improved Perlin with better isotropy (no directional bias)
// 3. **Worley Noise**: Cellular/voronoi patterns based on distance to random points
//
// All noise functions support **Fractal Brownian Motion (fBm)** which layers multiple
// octaves at different frequencies to create natural-looking detail.

// ============================================================================
// HASH FUNCTIONS
// ============================================================================
// Hash functions convert integer coordinates into pseudo-random numbers.
// These are deterministic (same input always gives same output) which is
// critical for coherent noise.

/// 2D integer hash function with seed.
///
/// Takes integer coordinates (x, y) and a seed, produces a pseudo-random u32.
/// This uses a **multiplicative hash** with prime numbers for good distribution.
///
/// How it works:
/// 1. Start with seed
/// 2. Mix in x coordinate with prime multiply + add
/// 3. Mix in y coordinate with different prime
/// 4. XOR-shift and multiply for avalanche effect (changing 1 bit affects all bits)
///
/// Properties:
/// - Deterministic: same (x,y,seed) always gives same result
/// - Uniform distribution: output values evenly distributed
/// - Good avalanche: nearby inputs produce very different outputs
uint hash_2d(int x, int y, uint seed) {
    uint h = seed;
    h = h * 374761393u + uint(x);   // Prime multiply-add with x
    h = h * 668265263u + uint(y);   // Prime multiply-add with y
    h ^= h >> 13;                    // XOR-shift right (avalanche)
    h *= 1274126177u;                // Prime multiply (spread bits)
    h ^= h >> 16;                    // Final avalanche
    return h;
}

/// Converts a hash value to a float in range [0, 1].
///
/// Divides by maximum u32 value to normalize to [0, 1].
float random_float(uint hash) {
    return float(hash) / float(0xFFFFFFFFu);
}

// ============================================================================
// PERLIN NOISE HELPERS
// ============================================================================

/// Fade function for smooth interpolation (6t⁵ - 15t⁴ + 10t³).
///
/// This is Ken Perlin's improved fade function. It provides C2-continuous
/// interpolation (second derivative is continuous) which eliminates visible
/// grid artifacts.
///
/// Properties:
/// - f(0) = 0, f(1) = 1 (correct endpoints)
/// - f'(0) = f'(1) = 0 (zero first derivative at endpoints)
/// - f''(0) = f''(1) = 0 (zero second derivative at endpoints)
///
/// Why this matters:
/// Without smooth interpolation, you'd see a visible grid pattern where
/// noise cells meet. This function makes the transitions imperceptible.
float fade(float t) {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

/// Computes gradient dot product for Perlin noise.
///
/// Takes a hash value and position, returns gradient contribution.
///
/// Perlin noise works by:
/// 1. Place random gradients at grid corners
/// 2. Compute dot product of gradient with distance vector
/// 3. Interpolate the results
///
/// This function selects one of 8 gradient directions based on hash,
/// then computes the dot product with (x, y) offset.
float grad_2d(uint hash, float x, float y) {
    uint h = hash & 7u;                       // Select one of 8 gradients
    float u = (h < 4u) ? x : y;               // Select major axis
    float v = (h < 4u) ? y : x;               // Select minor axis
    u = ((h & 1u) == 0u) ? u : -u;            // Random sign for u
    v = ((h & 2u) == 0u) ? v : -v;            // Random sign for v
    return u + v;                             // Gradient dot product
}

// ============================================================================
// PERLIN NOISE
// ============================================================================

/// 2D Perlin noise function.
///
/// Returns noise value in range approximately [-1, 1] (typically [-0.7, 0.7]).
///
/// **How Perlin noise works:**
/// 1. Find the 4 integer grid corners surrounding (x, y)
/// 2. Hash each corner to get a gradient direction
/// 3. Compute gradient dot products with offset vectors
/// 4. Interpolate using smooth fade function
///
/// **Why it's useful:**
/// - Smooth, continuous noise (no sharp edges)
/// - Natural-looking when layered with fBm
/// - Good for clouds, terrain, organic textures
///
/// **Parameters:**
/// - x, y: Continuous coordinates (any float values)
/// - seed: Random seed for reproducibility
float perlin_noise(float x, float y, uint seed) {
    // Find integer grid cell containing (x, y)
    int xi = int(floor(x));
    int yi = int(floor(y));
    
    // Fractional position within cell [0, 1]
    float xf = x - floor(x);
    float yf = y - floor(y);
    
    // Compute smooth interpolation curves (fade function)
    float u = fade(xf);
    float v = fade(yf);
    
    // Compute gradient contributions from 4 corners
    // Each corner has a random gradient, we compute dot product with offset
    float aa = grad_2d(hash_2d(xi,     yi,     seed), xf,       yf);        // Bottom-left
    float ba = grad_2d(hash_2d(xi + 1, yi,     seed), xf - 1.0, yf);        // Bottom-right
    float ab = grad_2d(hash_2d(xi,     yi + 1, seed), xf,       yf - 1.0);  // Top-left
    float bb = grad_2d(hash_2d(xi + 1, yi + 1, seed), xf - 1.0, yf - 1.0);  // Top-right
    
    // Bilinear interpolation of the 4 corner contributions
    float x1 = mix(aa, ba, u);  // Interpolate bottom edge
    float x2 = mix(ab, bb, u);  // Interpolate top edge
    return mix(x1, x2, v);      // Interpolate between edges
}

// ============================================================================
// SIMPLEX NOISE HELPERS
// ============================================================================

/// Computes contribution from a single simplex corner.
///
/// Simplex noise uses a different grid (triangles instead of squares) which
/// provides better isotropy and is more computationally efficient.
///
/// Each corner contributes a radial falloff: (0.5 - r²)⁴ × gradient
/// where r is distance from corner.
float simplex_contrib(float x, float y, uint hash) {
    float t = 0.5 - x * x - y * y;  // Radial falloff
    if (t < 0.0) return 0.0;         // Outside contribution radius
    t = t * t;                       // (t²)²
    return t * t * grad_2d(hash, x, y);
}

// ============================================================================
// SIMPLEX NOISE
// ============================================================================

/// 2D Simplex noise function (improved Perlin).
///
/// Returns noise value in range approximately [-1, 1].
///
/// **Advantages over Perlin:**
/// - Better isotropy (no directional bias)
/// - More computationally efficient (3 corners vs 4)
/// - Smoother appearance at higher frequencies
///
/// **How it works:**
/// 1. Skew input space to align with simplex grid (triangular)
/// 2. Find which simplex (triangle) contains the point
/// 3. Compute contributions from 3 triangle corners
/// 4. Sum contributions (each has radial falloff)
///
/// **Uses:**
/// - Preferred over Perlin for most applications
/// - Excellent for natural textures
/// - Fire, clouds, marble, wood grain
float simplex_noise(float x, float y, uint seed) {
    // Skewing factors for 2D simplex grid
    const float F2 = 0.366025404;  // (sqrt(3) - 1) / 2
    const float G2 = 0.211324865;  // (3 - sqrt(3)) / 6
    
    // Skew input space to determine which simplex cell we're in
    float s = (x + y) * F2;
    float xs = x + s;
    float ys = y + s;
    float i = floor(xs);
    float j = floor(ys);
    
    // Unskew cell origin back to (x,y) space
    float t = (i + j) * G2;
    float x0 = x - (i - t);  // Distance from cell origin
    float y0 = y - (j - t);
    
    // Determine which simplex we're in (upper or lower triangle)
    float i1 = (x0 > y0) ? 1.0 : 0.0;  // Offsets for second corner
    float j1 = (x0 > y0) ? 0.0 : 1.0;
    
    // Offsets for three simplex corners in (x,y) space
    float x1 = x0 - i1 + G2;           // Second corner
    float y1 = y0 - j1 + G2;
    float x2 = x0 - 1.0 + 2.0 * G2;    // Third corner
    float y2 = y0 - 1.0 + 2.0 * G2;
    
    int ii = int(i);
    int jj = int(j);
    
    // Calculate contributions from three corners
    float n0 = simplex_contrib(x0, y0, hash_2d(ii,              jj,              seed));
    float n1 = simplex_contrib(x1, y1, hash_2d(ii + int(i1),   jj + int(j1),   seed));
    float n2 = simplex_contrib(x2, y2, hash_2d(ii + 1,         jj + 1,         seed));
    
    // Sum and scale to approximately [-1, 1]
    return 40.0 * (n0 + n1 + n2);
}

// ============================================================================
// WORLEY (CELLULAR) NOISE
// ============================================================================

/// Worley/Cellular noise based on distance to nearest random point.
///
/// Returns distance in range [0, 1+].
///
/// **How it works:**
/// 1. Divide space into cells (like a grid)
/// 2. Place random point in each cell
/// 3. For any query point, find distance to nearest random point
/// 4. Distance becomes the noise value
///
/// **Characteristics:**
/// - Creates cellular/voronoi patterns
/// - Sharp boundaries between cells
/// - Natural for stone, scales, honeycomb, water caustics
///
/// **cell_size parameter:**
/// - Larger = bigger cells, smoother patterns
/// - Smaller = more cells, finer detail
/// - Typically 1.0 for standard patterns
///
/// **Usage notes:**
/// - Output is distance, not gradient (different from Perlin/Simplex)
/// - Can use different distance metrics (Euclidean, Manhattan, Chebyshev)
/// - Can use 2nd-nearest distance for different patterns
float worley_noise(float x, float y, uint seed, float cell_size) {
    // Scale coordinates by cell size
    x = x / cell_size;
    y = y / cell_size;
    
    // Find integer cell coordinates
    int cell_x = int(floor(x));
    int cell_y = int(floor(y));
    
    float min_dist = 1000000.0;  // Start with huge distance
    
    // Check 3×3 neighborhood of cells (current + 8 neighbors)
    // We need to check neighbors because nearest point might be in adjacent cell
    for (int dy = -1; dy <= 1; dy++) {
        for (int dx = -1; dx <= 1; dx++) {
            int neighbor_x = cell_x + dx;
            int neighbor_y = cell_y + dy;
            
            // Generate random point position within this cell
            uint hash = hash_2d(neighbor_x, neighbor_y, seed);
            float point_x = float(neighbor_x) + random_float(hash);
            float point_y = float(neighbor_y) + random_float(hash * 2654435761u);  // Different hash for y
            
            // Compute Euclidean distance to this point
            float diff_x = x - point_x;
            float diff_y = y - point_y;
            float dist = sqrt(diff_x * diff_x + diff_y * diff_y);
            
            // Track minimum distance
            min_dist = min(min_dist, dist);
        }
    }
    
    // Clamp to [0, 1] (distances are typically < 1.0 but can exceed)
    return min(min_dist, 1.0);
}

// ============================================================================
// FRACTAL BROWNIAN MOTION (fBm)
// ============================================================================
//
// fBm layers multiple octaves of noise at different frequencies to create
// natural-looking detail. Each octave has:
// - 2× frequency (lacunarity = 2.0)
// - 0.5× amplitude (persistence = 0.5)
//
// More octaves = more fine detail but slower computation.
// Typical values: 3-6 octaves for most textures.
//
// The result is normalized by dividing by sum of amplitudes, ensuring
// output stays in approximately the same range as single-octave noise.

/// fBm using Perlin noise.
///
/// Layers multiple octaves of Perlin noise for natural-looking detail.
///
/// **Parameters:**
/// - uv: 2D coordinates
/// - seed: Random seed (different seed per octave: seed+i)
/// - octaves: Number of detail layers (typically 3-6)
/// - persistence: Amplitude decay per octave (typically 0.5)
/// - lacunarity: Frequency multiplier per octave (typically 2.0)
float fbm_perlin_noise(vec2 uv, uint seed, int octaves, float persistence, float lacunarity) {
    float value = 0.0;       // Accumulated noise value
    float amplitude = 1.0;   // Current octave amplitude
    float frequency = 1.0;   // Current octave frequency
    float max_value = 0.0;   // For normalization
    
    // Layer octaves of noise
    for (int i = 0; i < octaves; i++) {
        // Sample noise at current frequency, scale by amplitude
        value += perlin_noise(uv.x * frequency, uv.y * frequency, seed + uint(i)) * amplitude;
        max_value += amplitude;
        
        // Decay amplitude and increase frequency for next octave
        amplitude *= persistence;  // Each octave is quieter
        frequency *= lacunarity;   // Each octave is higher frequency
    }
    
    // Normalize by sum of amplitudes
    return value / max_value;
}

/// fBm using Simplex noise.
///
/// Same as Perlin fBm but uses Simplex noise for better quality.
/// Preferred over Perlin fBm for most applications.
float fbm_simplex_noise(vec2 uv, uint seed, int octaves, float persistence, float lacunarity) {
    float value = 0.0;
    float amplitude = 1.0;
    float frequency = 1.0;
    float max_value = 0.0;
    
    for (int i = 0; i < octaves; i++) {
        value += simplex_noise(uv.x * frequency, uv.y * frequency, seed + uint(i)) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }
    
    return value / max_value;
}

/// fBm using Worley noise.
///
/// Layers Worley noise for detailed cellular patterns.
/// Useful for complex organic textures like cracked earth, scales, etc.
float fbm_worley_noise(vec2 uv, uint seed, int octaves, float persistence, float lacunarity) {
    float value = 0.0;
    float amplitude = 1.0;
    float frequency = 1.0;
    float max_value = 0.0;
    
    for (int i = 0; i < octaves; i++) {
        // Note: Worley noise takes frequency-scaled UVs directly
        value += worley_noise(uv.x * frequency, uv.y * frequency, seed + uint(i), 1.0) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }
    
    return value / max_value;
}
