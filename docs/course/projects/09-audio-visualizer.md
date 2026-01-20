# Project 09: Audio-Reactive Visualizer

**Difficulty**: Intermediate  
**Estimated Time**: 2-3 weeks  
**Core Learning**: Audio processing, FFT analysis, real-time visualization, synchronization

## Overview

Build an audio-reactive visualizer that responds to music with dynamic visual effects. This project teaches audio input processing, frequency analysis with FFT, beat detection, visual effect composition, and audio-visual synchronization techniques.

### Learning Objectives

- Process audio input (file or microphone)
- Perform FFT (Fast Fourier Transform) for frequency analysis
- Implement beat detection algorithms
- Synchronize visuals to audio features
- Create procedural animations driven by audio
- Optimize real-time audio-visual performance

## Feature Requirements

### Core Features (Minimum Viable)

1. **Audio Playback**
   - Load and play audio files (MP3, WAV, OGG)
   - Playback controls (play, pause, stop, seek)
   - Volume control
   - Time display (current/total)

2. **Frequency Analysis**
   - FFT (Fast Fourier Transform) computation
   - Visualize frequency spectrum (bars or line)
   - Separate frequency bands (bass, mid, treble)
   - Real-time updates (60 FPS)

3. **Basic Visualizations**
   - Spectrum analyzer (frequency bars)
   - Waveform display (time domain)
   - Circle visualizer (radius responds to volume)
   - Color changes based on frequency content

4. **Audio Reactivity**
   - Objects scale with amplitude
   - Colors shift with frequency
   - Particle emission rate driven by beat
   - Camera shake/movement on bass hits

### Extended Features (Recommended)

5. **Beat Detection**
   - Detect kick drum hits (low frequency peaks)
   - Detect snare/hi-hat (mid/high frequency)
   - Visual flash or pulse on beat
   - BPM (beats per minute) estimation
   - Beat-synchronized animations

6. **Advanced Visualizations**
   - 3D spectrum (height-mapped terrain)
   - Particle systems reacting to frequency bands
   - Geometric patterns (kaleidoscope, fractals)
   - Tunnel/wormhole effect
   - Mesh deformation based on audio

7. **Multiple Visualization Modes**
   - Switch between different visualizer styles
   - Smooth transitions between modes
   - Preset library (different looks)
   - User-configurable parameters (sensitivity, colors)

### Stretch Goals

8. **Advanced Audio Analysis**
   - Onset detection (transient events)
   - Pitch detection (melody tracking)
   - Tempo/rhythm analysis
   - Audio feature extraction (MFCC, spectral centroid)

9. **Export and Sharing**
   - Record visualization to video (MP4)
   - Screenshot capture
   - VJ mode (live performance controls)
   - Shader-based visualizers (GPU compute)

## Architecture Guidance

### System Components

```
AudioVisualizer
├── AudioEngine
│   ├── AudioPlayer (playback control)
│   ├── AudioBuffer (PCM data)
│   └── AudioDecoder (MP3/WAV/OGG)
├── AudioAnalyzer
│   ├── FFTProcessor
│   ├── FrequencyBands (bass, mid, treble)
│   ├── BeatDetector
│   └── FeatureExtractor
├── VisualizationEngine
│   ├── SpectrumVisualizer
│   ├── WaveformVisualizer
│   ├── ParticleVisualizer
│   └── GeometricVisualizer
├── AnimationController
│   ├── BeatAnimator
│   ├── ParameterSmoother
│   └── TransitionManager
└── UI
    ├── PlaybackControls
    ├── VisualizerSelector
    └── ParameterEditor
```

### Data Structures

**Audio Buffer**
```
AudioBuffer:
  - samples: array of floats (PCM data, -1.0 to 1.0)
  - sample_rate: int (e.g., 44100 Hz)
  - channels: int (1 = mono, 2 = stereo)
  - current_position: int (sample index)

Methods:
  - get_samples(offset, count) -> array
  - get_stereo_mixed() -> array (mix to mono)
  - advance(sample_count)
```

**FFT Result**
```
FFTResult:
  - frequencies: array of floats (magnitude per bin)
  - bin_count: int (typically 512, 1024, or 2048)
  - frequency_resolution: float (Hz per bin)

Methods:
  - get_magnitude(frequency_hz) -> float
  - get_band(low_hz, high_hz) -> float (average or sum)

Frequency Bands:
  - Sub-bass: 20-60 Hz
  - Bass: 60-250 Hz
  - Low-mid: 250-500 Hz
  - Mid: 500-2000 Hz
  - High-mid: 2000-4000 Hz
  - Treble: 4000-20000 Hz
```

**Beat Detector**
```
BeatDetector:
  - energy_history: circular buffer of floats
  - history_size: int (e.g., 43 frames ≈ 1 second at 43 FPS)
  - threshold_multiplier: float (e.g., 1.5)
  - last_beat_time: float
  - min_beat_interval: float (e.g., 0.1 seconds)

Methods:
  - process(fft_result, current_time) -> bool (beat detected)
  - get_beat_strength() -> float (0-1)

Algorithm:
  instant_energy = sum(fft_result.frequencies)
  average_energy = mean(energy_history)
  
  is_beat = instant_energy > average_energy * threshold_multiplier
            and (current_time - last_beat_time) > min_beat_interval
  
  energy_history.push(instant_energy)
```

**Visualizer Parameters**
```
VisualizerParams:
  - sensitivity: float (amplification factor)
  - smoothing: float (0-1, temporal smoothing)
  - color_palette: array of colors
  - color_mode: Frequency | Energy | Beat
  - rotation_speed: float
  - scale_min: float
  - scale_max: float

ReactiveValue:
  - current: float
  - target: float
  - smoothing: float

Methods:
  - set_target(value)
  - update(delta_time):
      current = lerp(current, target, smoothing * delta_time)
  - get() -> float
```

### FFT Processing Pipeline

```
process_audio_frame():
  # 1. Get audio samples
  sample_count = 1024  # Must be power of 2
  samples = audio_buffer.get_samples(current_position, sample_count)
  
  # 2. Apply window function (reduce spectral leakage)
  windowed_samples = apply_hanning_window(samples)
  
  # 3. Perform FFT
  fft_result = fft(windowed_samples)
  
  # 4. Compute magnitudes
  magnitudes = []
  for i in 0..sample_count/2:  # Only first half (Nyquist)
    real = fft_result[i].real
    imag = fft_result[i].imag
    magnitude = sqrt(real*real + imag*imag)
    magnitudes.push(magnitude)
  
  # 5. Convert to decibels (optional, for better visualization)
  db_magnitudes = []
  for mag in magnitudes:
    db = 20 * log10(mag + epsilon)
    db_magnitudes.push(db)
  
  return db_magnitudes
```

**Window Functions**
```
hanning_window(samples):
  n = samples.length
  for i in 0..n:
    window = 0.5 * (1 - cos(2 * PI * i / (n - 1)))
    samples[i] *= window
  return samples

# Alternatives: Hamming, Blackman, etc.
```

### Beat Detection Algorithm

**Energy-Based (Simple)**
```
detect_beat(fft_result, history_buffer):
  # Focus on bass frequencies (60-250 Hz)
  bass_energy = 0
  for freq in 60..250 Hz:
    bass_energy += fft_result.get_magnitude(freq)
  
  # Calculate average energy over history
  average_energy = mean(history_buffer)
  
  # Detect beat if current energy significantly exceeds average
  variance = variance(history_buffer)
  threshold = average_energy + threshold_multiplier * sqrt(variance)
  
  is_beat = bass_energy > threshold
  
  # Update history
  history_buffer.push(bass_energy)
  
  return is_beat
```

**Onset Detection (Advanced)**
```
detect_onset(current_spectrum, previous_spectrum):
  # Spectral flux: measure change in spectrum
  spectral_flux = 0
  for i in 0..spectrum_size:
    diff = max(0, current_spectrum[i] - previous_spectrum[i])
    spectral_flux += diff
  
  # Peak picking in spectral flux
  if spectral_flux > peak_threshold and is_local_maximum:
    return true
  
  return false
```

### Visualization Examples

**Spectrum Analyzer**
```
render_spectrum_bars(fft_result):
  bar_count = 64
  bar_width = screen_width / bar_count
  
  for i in 0..bar_count:
    # Map FFT bins to bars (logarithmic frequency scale)
    bin_start = freq_to_bin(i * frequency_range / bar_count)
    bin_end = freq_to_bin((i + 1) * frequency_range / bar_count)
    
    # Average magnitude in bin range
    magnitude = average(fft_result[bin_start..bin_end])
    
    # Scale to screen height
    bar_height = magnitude * sensitivity * screen_height
    
    # Render bar
    x = i * bar_width
    y = screen_height - bar_height
    color = get_color_for_frequency(i)
    
    draw_rect(x, y, bar_width, bar_height, color)
```

**Audio-Reactive Particles**
```
update_particles(fft_result, beat_detected):
  bass = fft_result.get_band(60, 250)
  mid = fft_result.get_band(250, 2000)
  treble = fft_result.get_band(2000, 20000)
  
  # Spawn rate based on energy
  spawn_rate = bass * 100  # particles per second
  
  # Particle size based on mid frequencies
  particle_size = mid * 5.0
  
  # Color based on treble
  hue = treble * 360
  particle_color = hsv_to_rgb(hue, 1.0, 1.0)
  
  # Burst on beat
  if beat_detected:
    spawn_burst(50, particle_color)
```

**3D Mesh Deformation**
```
update_mesh_vertices(mesh, fft_result):
  for vertex in mesh.vertices:
    # Map vertex position to frequency
    angle = atan2(vertex.z, vertex.x)
    freq_index = (angle + PI) / (2 * PI) * fft_result.length
    
    # Get magnitude at frequency
    magnitude = fft_result[freq_index]
    
    # Deform outward
    direction = normalize(vertex.position)
    offset = direction * magnitude * deform_scale
    
    vertex.position = original_position + offset
```

## Milestone Plan

### Milestone 1: Audio Playback (Week 1, Days 1-2)

**Goal**: Load and play audio files

**Tasks**:
- Integrate audio library (Kira, rodio, miniaudio, etc.)
- Load audio file (WAV or MP3)
- Implement playback controls (play/pause)
- Add volume slider
- Display playback time
- Visualize basic waveform (time domain)

**Deliverable**: Audio player with waveform

### Milestone 2: FFT Implementation (Week 1, Days 3-4)

**Goal**: Compute and display frequency spectrum

**Tasks**:
- Implement or integrate FFT library
- Capture audio samples in real-time
- Apply window function (Hanning)
- Compute FFT each frame
- Display spectrum as bars
- Tune parameters (sample count, sensitivity)

**Deliverable**: Real-time spectrum analyzer

### Milestone 3: Frequency Bands (Week 1, Days 5-6)

**Goal**: Separate bass, mid, treble

**Tasks**:
- Define frequency band ranges
- Calculate energy per band
- Display separate bars for each band
- Color-code bands (red=bass, green=mid, blue=treble)
- Smooth band values over time (low-pass filter)

**Deliverable**: Multi-band spectrum display

### Milestone 4: Beat Detection (Week 1, Day 7 - Week 2, Day 1)

**Goal**: Detect beats in music

**Tasks**:
- Implement energy-based beat detection
- Maintain energy history buffer
- Detect peaks above threshold
- Visual feedback on beat (flash or pulse)
- Tune threshold and sensitivity
- Display BPM estimate

**Deliverable**: Beat-reactive visuals

### Milestone 5: 3D Visualization (Week 2, Days 2-4)

**Goal**: Audio-reactive 3D graphics

**Tasks**:
- Create 3D scene with objects
- Scale objects with audio energy
- Rotate based on beat
- Change colors with frequency
- Add particle system (emit on beat)
- Camera movement (dolly with bass)

**Deliverable**: 3D audio-reactive scene

### Milestone 6: Multiple Visualizers (Week 2, Days 5-7)

**Goal**: Different visualization styles

**Tasks**:
- Implement spectrum bars visualizer
- Implement circle/radial visualizer
- Implement 3D terrain heightmap visualizer
- Implement geometric pattern visualizer
- Add visualizer switching (hotkeys or UI)
- Smooth transitions between modes

**Deliverable**: Library of visualizers

### Milestone 7: Polish and Presets (Week 3, Days 1+)

**Goal**: Polished, user-friendly visualizer

**Tasks**:
- Add parameter tweaking UI (sliders for sensitivity, etc.)
- Create visual presets (save/load configurations)
- Add file browser (load different songs)
- Implement fullscreen mode
- Add post-processing effects (bloom, blur)
- Performance optimization
- Export video (optional)

**Deliverable**: Complete audio visualizer app

## Technical Challenges

### Challenge 1: Audio Sample Synchronization

**Problem**: Visual updates must sync with audio playback

**Approach**:
- Query audio playback position each frame
- Extract samples at current position
- Account for audio buffer latency
- Use ring buffer for smooth sample access

**Implementation**:
```
update_visualization():
  # Get current playback position (in samples)
  current_sample = audio_engine.get_playback_position()
  
  # Extract samples for FFT
  fft_samples = audio_buffer.get_samples(current_sample, fft_size)
  
  # Process FFT
  fft_result = compute_fft(fft_samples)
  
  # Update visuals
  update_visualizers(fft_result)
```

### Challenge 2: FFT Performance

**Problem**: FFT computation can be expensive for large sample sizes

**Approach**:
- Use optimized FFT library (FFTW, rustfft)
- Limit FFT size (1024 or 2048 samples usually sufficient)
- Compute FFT on separate thread (if necessary)
- Reuse FFT plans/buffers
- Consider GPU FFT for very large sizes

### Challenge 3: Frequency Band Mapping

**Problem**: Linear frequency scale doesn't match human perception

**Approach**:
- Use logarithmic frequency scale
- Map FFT bins to perceptual bands (mel scale)
- Group bins exponentially (more detail in low frequencies)

**Logarithmic Mapping**:
```
map_to_log_scale(fft_result, bar_count):
  min_freq = 20  # Hz
  max_freq = 20000  # Hz
  
  bars = []
  for i in 0..bar_count:
    # Logarithmic frequency interpolation
    f_low = min_freq * pow(max_freq / min_freq, i / bar_count)
    f_high = min_freq * pow(max_freq / min_freq, (i + 1) / bar_count)
    
    bin_low = freq_to_bin(f_low)
    bin_high = freq_to_bin(f_high)
    
    magnitude = average(fft_result[bin_low..bin_high])
    bars.push(magnitude)
  
  return bars
```

### Challenge 4: Temporal Smoothing

**Problem**: Raw FFT values flicker too much

**Approach**:
- Apply exponential smoothing (low-pass filter)
- Use different smoothing factors for different bands
- Smooth attack and decay separately (fast attack, slow decay)

**Smoothing Algorithm**:
```
ReactiveValue:
  current: float = 0
  
  update(target, delta_time):
    if target > current:
      # Fast attack
      smoothing = 0.9
    else:
      # Slow decay
      smoothing = 0.95
    
    current = lerp(current, target, 1.0 - smoothing)
```

### Challenge 5: Beat Detection False Positives

**Problem**: Noise or sustained sounds trigger false beats

**Approach**:
- Minimum beat interval (prevent beats faster than humanly possible)
- Variance threshold (require significant energy change)
- Frequency-specific detection (focus on kick drum frequencies)
- Adaptive thresholding (adjust based on recent history)

## Reference Implementations

### Praxis Engine (Rust)
- **Files**: `examples/audio_demo.rs`, `examples/audio_simple.rs`
- **Crates**: `praxis_audio`
- **Concepts**: Audio playback, spatial audio

### Other Engines/Frameworks

**Processing (Java/JavaScript)**
- Library: Minim (Java), p5.sound.js (JavaScript)
- Example: Countless audio visualizer sketches
- Pattern: Simple FFT API, creative coding friendly

**Three.js + Web Audio API (JavaScript)**
- Tutorial: "Audio Visualizer with Three.js"
- APIs: `AudioContext`, `AnalyserNode`, Three.js for rendering
- Example: Many CodePen/Shadertoy examples

**Unity (C#)**
- Package: Audio visualization packages (Asset Store)
- API: `AudioSource.GetSpectrumData()`, `AudioSource.GetOutputData()`
- Tutorial: "Audio Spectrum" (various YouTube)

**Shadertoy (GLSL)**
- Platform: Shadertoy.com
- Examples: Search "audio" for shader-based visualizers
- Pattern: Sound texture input to fragment shader

**Bevy (Rust)**
- Plugin: `bevy_kira_audio` + custom visualization
- Pattern: ECS-based audio reactive systems

**VJ Software**
- Examples: Resolume, VDMX (proprietary but good reference)
- Features: Professional audio-reactive VFX

## Extension Ideas

### Beginner Extensions
- Microphone input (live visualization)
- Save visualizer settings as presets
- Multiple color palettes
- Fullscreen toggle

### Intermediate Extensions
- MIDI input support (control parameters with MIDI controller)
- Stereo separation (left/right channel visualization)
- Audio effects (echo, reverb visualization)
- Lyrics/metadata display

### Advanced Extensions
- Machine learning (audio classification, mood detection)
- VR/AR support (immersive visualizer)
- Multi-user sync (synchronized visualizers across devices)
- Shader-based visualizers (entirely GPU-driven)

## Success Criteria

Your audio visualizer should:

1. ✅ Play audio files smoothly without glitches
2. ✅ Display real-time frequency spectrum at 60 FPS
3. ✅ React visibly to bass, mid, and treble frequencies
4. ✅ Detect beats accurately for typical music
5. ✅ Provide multiple interesting visualization styles
6. ✅ Allow user customization (sensitivity, colors)
7. ✅ Feel synchronized with music (no noticeable lag)

## Assessment Rubric

| Category | Beginner | Intermediate | Advanced |
|----------|----------|--------------|----------|
| **Audio Processing** | Playback, basic FFT | Frequency bands, smoothing | Beat detection, onset, advanced features |
| **Visualizations** | 1-2 simple visuals | 3-4 varied styles | 5+ polished, unique visualizers |
| **Reactivity** | Visuals respond to volume | Responds to frequency bands | Beat-sync, complex audio features |
| **Performance** | 30 FPS, occasional stutters | 60 FPS stable | 60 FPS, optimized, GPU-accelerated |

## Common Pitfalls

1. **No Window Function**: Causes spectral leakage, noisy FFT
2. **Linear Frequency Scale**: Doesn't match perception, use logarithmic
3. **No Temporal Smoothing**: Flickering, chaotic visuals
4. **Audio Thread Blocking**: Never block audio thread with heavy computation
5. **Fixed Sensitivity**: Different songs need different amplification
6. **Ignoring Nyquist**: Only use first half of FFT output
7. **Synchronization Lag**: Query playback position, don't assume constant rate

## Next Steps

After completing this project, you're ready for:
- **Project 06**: Particle Effects System (audio-reactive particle emitters)
- **Project 10**: Mini Game Engine (integrate audio subsystem with visualization)
- Advanced: Build VJ software or music production visualizer
