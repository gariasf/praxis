// Noise function implementations for GPU compute shaders
// These implementations match the CPU versions in noise.rs

uint hash_2d(int x, int y, uint seed) {
    uint h = seed;
    h = h * 374761393u + uint(x);
    h = h * 668265263u + uint(y);
    h ^= h >> 13;
    h *= 1274126177u;
    h ^= h >> 16;
    return h;
}

float random_float(uint hash) {
    return float(hash) / float(0xFFFFFFFFu);
}

float fade(float t) {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

float grad_2d(uint hash, float x, float y) {
    uint h = hash & 7u;
    float u = (h < 4u) ? x : y;
    float v = (h < 4u) ? y : x;
    u = ((h & 1u) == 0u) ? u : -u;
    v = ((h & 2u) == 0u) ? v : -v;
    return u + v;
}

float perlin_noise(float x, float y, uint seed) {
    int xi = int(floor(x));
    int yi = int(floor(y));
    float xf = x - floor(x);
    float yf = y - floor(y);
    
    float u = fade(xf);
    float v = fade(yf);
    
    float aa = grad_2d(hash_2d(xi, yi, seed), xf, yf);
    float ba = grad_2d(hash_2d(xi + 1, yi, seed), xf - 1.0, yf);
    float ab = grad_2d(hash_2d(xi, yi + 1, seed), xf, yf - 1.0);
    float bb = grad_2d(hash_2d(xi + 1, yi + 1, seed), xf - 1.0, yf - 1.0);
    
    float x1 = mix(aa, ba, u);
    float x2 = mix(ab, bb, u);
    return mix(x1, x2, v);
}

float simplex_contrib(float x, float y, uint hash) {
    float t = 0.5 - x * x - y * y;
    if (t < 0.0) return 0.0;
    t = t * t;
    return t * t * grad_2d(hash, x, y);
}

float simplex_noise(float x, float y, uint seed) {
    const float F2 = 0.366025404;
    const float G2 = 0.211324865;
    
    float s = (x + y) * F2;
    float xs = x + s;
    float ys = y + s;
    float i = floor(xs);
    float j = floor(ys);
    
    float t = (i + j) * G2;
    float x0 = x - (i - t);
    float y0 = y - (j - t);
    
    float i1 = (x0 > y0) ? 1.0 : 0.0;
    float j1 = (x0 > y0) ? 0.0 : 1.0;
    
    float x1 = x0 - i1 + G2;
    float y1 = y0 - j1 + G2;
    float x2 = x0 - 1.0 + 2.0 * G2;
    float y2 = y0 - 1.0 + 2.0 * G2;
    
    int ii = int(i);
    int jj = int(j);
    
    float n0 = simplex_contrib(x0, y0, hash_2d(ii, jj, seed));
    float n1 = simplex_contrib(x1, y1, hash_2d(ii + int(i1), jj + int(j1), seed));
    float n2 = simplex_contrib(x2, y2, hash_2d(ii + 1, jj + 1, seed));
    
    return 40.0 * (n0 + n1 + n2);
}

float worley_noise(float x, float y, uint seed, float cell_size) {
    x = x / cell_size;
    y = y / cell_size;
    
    int cell_x = int(floor(x));
    int cell_y = int(floor(y));
    
    float min_dist = 1000000.0;
    
    for (int dy = -1; dy <= 1; dy++) {
        for (int dx = -1; dx <= 1; dx++) {
            int neighbor_x = cell_x + dx;
            int neighbor_y = cell_y + dy;
            
            uint hash = hash_2d(neighbor_x, neighbor_y, seed);
            float point_x = float(neighbor_x) + random_float(hash);
            float point_y = float(neighbor_y) + random_float(hash * 2654435761u);
            
            float diff_x = x - point_x;
            float diff_y = y - point_y;
            float dist = sqrt(diff_x * diff_x + diff_y * diff_y);
            
            min_dist = min(min_dist, dist);
        }
    }
    
    return min(min_dist, 1.0);
}

float fbm_perlin_noise(vec2 uv, uint seed, int octaves, float persistence, float lacunarity) {
    float value = 0.0;
    float amplitude = 1.0;
    float frequency = 1.0;
    float max_value = 0.0;
    
    for (int i = 0; i < octaves; i++) {
        value += perlin_noise(uv.x * frequency, uv.y * frequency, seed + uint(i)) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }
    
    return value / max_value;
}

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

float fbm_worley_noise(vec2 uv, uint seed, int octaves, float persistence, float lacunarity) {
    float value = 0.0;
    float amplitude = 1.0;
    float frequency = 1.0;
    float max_value = 0.0;
    
    for (int i = 0; i < octaves; i++) {
        value += worley_noise(uv.x * frequency, uv.y * frequency, seed + uint(i), 1.0) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }
    
    return value / max_value;
}
