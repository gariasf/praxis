//! PROTOTYPE (ticket 800), throwaway: the controls of the LOD transition
//! prototype, read from the environment at first use and changed by the
//! window's keys, which reach the shaders in the camera uniform's spare
//! words.
//!
//! `PRAXIS_PROTOTYPE_LOD`: `off` (every Instance at LOD 0), `t1` (hard
//! switch), `t3-fixed` (per-sample cross-fade, one count per Instance),
//! `t3-pixel` (the count dithered per pixel by the Peters mask), `t2`
//! (per-pixel dither of whole pixels). `PRAXIS_PROTOTYPE_FORCE_LOD`: a LOD
//! every Instance draws, clamped to its chain. `PRAXIS_PROTOTYPE_FORCE_SHARE`:
//! with a forced LOD, the share 0 to 255 it keeps against the next LOD.
//! `PRAXIS_PROTOTYPE_BAND`: the ratio of the projected LOD errors a fade
//! spans. `PRAXIS_PROTOTYPE_THRESHOLD`: the LOD error in pixels.
//! `PRAXIS_PROTOTYPE_TINT`: colours each LOD past 0.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

pub const MODES: [&str; 5] = ["off", "t1", "t3-fixed", "t3-pixel", "t2"];
pub const BANDS: [f32; 3] = [1.25, 1.5, 2.0];
const NO_FORCED_LOD: u32 = 15;
const NO_FORCED_SHARE: u32 = 256;

pub struct Controls {
    pub mode: AtomicU32,
    pub forced_lod: AtomicU32,
    pub forced_share: AtomicU32,
    pub band_bits: AtomicU32,
    pub threshold_bits: AtomicU32,
    pub tint: AtomicU32,
    /// PROTOTYPE (ticket 801): `THIN_MODES`' index, whether shadow rays
    /// leave out thin Meshes by width, and whether thin Meshes draw magenta.
    pub thin: AtomicU32,
    pub shadow_classes: AtomicU32,
    pub thin_tint: AtomicU32,
}

/// PROTOTYPE (ticket 801): how thin Meshes draw. `PRAXIS_PROTOTYPE_THIN`
/// names one; `PRAXIS_PROTOTYPE_SHADOW_CLASSES` starts with the shadow
/// classes on.
pub const THIN_MODES: [&str; 4] = ["cooked", "rounded", "dithered", "widened"];

static TARGET_HEIGHT: AtomicU32 = AtomicU32::new(1440);

pub fn set_target_height(height: u32) {
    TARGET_HEIGHT.store(height, Ordering::Relaxed);
}

pub fn thin_enabled() -> bool {
    std::env::var_os("PRAXIS_PROTOTYPE_THIN").is_some()
}

/// Whether the main pass takes the pipelines that write a sample mask.
pub fn carrier_enabled() -> bool {
    enabled() || thin_enabled()
}

const WIRE_THIN_WORD: u32 = 15 | (3 << 9);
const POST_THIN_WORD: u32 = 25 | (1 << 8) | (6 << 9);
/// The box Mesh: thin by each Instance's scale (`THIN_BOX`), four faces.
const BOX_THIN_WORD: u32 = 255 | (4 << 9);

fn is_unit_box(extent: [f32; 3]) -> bool {
    extent.iter().all(|side| (side - 1.0).abs() < 1e-6)
}

/// The thin word of the Mesh whose box is `bounds` (`common.wgsl`): `town`'s
/// wire and railing post, known by their boxes, as a Mesh's load has no name
/// for it. Zero for any other, and with the prototype off.
pub fn thin_word(bounds: [[f32; 3]; 2]) -> u32 {
    if !thin_enabled() {
        return 0;
    }
    let extent: [f32; 3] = std::array::from_fn(|axis| bounds[1][axis] - bounds[0][axis]);
    let word = if extent[0] > 10.0 && extent[1] < 1.0 && extent[2] < 0.03 {
        WIRE_THIN_WORD
    } else if extent[0] <= 0.0501 && (extent[1] - 1.0).abs() < 1e-3 && extent[2] <= 0.0501 {
        POST_THIN_WORD
    } else if is_unit_box(extent) {
        BOX_THIN_WORD
    } else {
        0
    };
    if word != 0 {
        tracing::info!(?extent, word, "PROTOTYPE thin Mesh");
    }
    word
}

/// The TLAS instance mask of the Mesh whose box is `bounds`: a thin Mesh's
/// own bit (`WIRE_RAY_MASK`, `POST_RAY_MASK`), every bit for any other.
pub fn ray_mask(bounds: [[f32; 3]; 2], transform: &[f32; 12]) -> u8 {
    static QUIET: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !thin_enabled() {
        return 0xff;
    }
    let extent: [f32; 3] = std::array::from_fn(|axis| bounds[1][axis] - bounds[0][axis]);
    let mask = if extent[0] > 10.0 && extent[1] < 1.0 && extent[2] < 0.03 {
        2
    } else if extent[0] <= 0.0501 && (extent[1] - 1.0).abs() < 1e-3 && extent[2] <= 0.0501 {
        4
    } else if is_unit_box(extent) {
        // A bar: one long axis, the other two scaled under 6 cm.
        let lengths: [f32; 3] = std::array::from_fn(|column| {
            (0..3)
                .map(|row| transform[row * 4 + column].powi(2))
                .sum::<f32>()
                .sqrt()
        });
        let short = lengths.iter().filter(|&&length| length < 0.06).count();
        let long = lengths.iter().filter(|&&length| length >= 0.5).count();
        if short == 2 && long == 1 { 4 } else { 0xff }
    } else {
        0xff
    };
    if mask != 0xff && !QUIET.swap(true, Ordering::Relaxed) {
        tracing::info!(mask, "PROTOTYPE thin Mesh in a TLAS");
    }
    mask
}

fn variable<Value: std::str::FromStr>(name: &str) -> Option<Value> {
    std::env::var(name).ok()?.parse().ok()
}

/// Whether the prototype is on at all: off, Meshes load no LOD chain and the
/// main pass keeps its pipelines.
pub fn enabled() -> bool {
    std::env::var_os("PRAXIS_PROTOTYPE_LOD").is_some()
}

pub fn controls() -> &'static Controls {
    static CONTROLS: OnceLock<Controls> = OnceLock::new();
    CONTROLS.get_or_init(|| {
        let mode = std::env::var("PRAXIS_PROTOTYPE_LOD")
            .ok()
            .and_then(|name| MODES.iter().position(|&mode| mode == name))
            .unwrap_or(0) as u32;
        Controls {
            mode: AtomicU32::new(mode),
            forced_lod: AtomicU32::new(
                variable("PRAXIS_PROTOTYPE_FORCE_LOD").unwrap_or(NO_FORCED_LOD),
            ),
            forced_share: AtomicU32::new(
                variable("PRAXIS_PROTOTYPE_FORCE_SHARE").unwrap_or(NO_FORCED_SHARE),
            ),
            band_bits: AtomicU32::new(
                variable("PRAXIS_PROTOTYPE_BAND")
                    .unwrap_or(1.5f32)
                    .to_bits(),
            ),
            threshold_bits: AtomicU32::new(
                variable("PRAXIS_PROTOTYPE_THRESHOLD")
                    .unwrap_or(1.0f32)
                    .to_bits(),
            ),
            tint: AtomicU32::new(u32::from(
                std::env::var_os("PRAXIS_PROTOTYPE_TINT").is_some(),
            )),
            thin: AtomicU32::new(
                std::env::var("PRAXIS_PROTOTYPE_THIN")
                    .ok()
                    .and_then(|name| THIN_MODES.iter().position(|&mode| mode == name))
                    .unwrap_or(0) as u32,
            ),
            shadow_classes: AtomicU32::new(u32::from(
                std::env::var_os("PRAXIS_PROTOTYPE_SHADOW_CLASSES").is_some(),
            )),
            thin_tint: AtomicU32::new(0),
        }
    })
}

/// The camera uniform's control word, threshold in pixels and band ratio.
pub fn uniform_words() -> (u32, f32, f32) {
    let controls = controls();
    let forced_share = controls.forced_share.load(Ordering::Relaxed);
    let control = controls.mode.load(Ordering::Relaxed)
        | (controls.forced_lod.load(Ordering::Relaxed).min(15) << 4)
        | (forced_share.min(255) << 8)
        | (u32::from(forced_share < NO_FORCED_SHARE) << 16)
        | (controls.tint.load(Ordering::Relaxed) << 17);
    let thin = (controls.thin.load(Ordering::Relaxed) << 18)
        | (controls.shadow_classes.load(Ordering::Relaxed) << 20)
        | (controls.thin_tint.load(Ordering::Relaxed) << 21);
    (
        control | thin,
        f32::from_bits(controls.threshold_bits.load(Ordering::Relaxed)),
        if thin_enabled() {
            TARGET_HEIGHT.load(Ordering::Relaxed) as f32
        } else {
            f32::from_bits(controls.band_bits.load(Ordering::Relaxed))
        },
    )
}

/// A key of the window: 0 to 4 pick the mode, B the next band, T the tint,
/// L the next forced LOD (none after 7), comma and full stop the threshold.
pub fn key(code: winit::keyboard::KeyCode) {
    use winit::keyboard::KeyCode;
    let controls = controls();
    match code {
        KeyCode::Digit0 => controls.mode.store(0, Ordering::Relaxed),
        KeyCode::Digit1 => controls.mode.store(1, Ordering::Relaxed),
        KeyCode::Digit2 => controls.mode.store(2, Ordering::Relaxed),
        KeyCode::Digit3 => controls.mode.store(3, Ordering::Relaxed),
        KeyCode::Digit4 => controls.mode.store(4, Ordering::Relaxed),
        KeyCode::KeyB => {
            let band = f32::from_bits(controls.band_bits.load(Ordering::Relaxed));
            let next = BANDS
                .iter()
                .position(|&listed| listed > band)
                .map_or(BANDS[0], |index| BANDS[index]);
            controls.band_bits.store(next.to_bits(), Ordering::Relaxed);
        }
        KeyCode::KeyT => {
            controls.tint.fetch_xor(1, Ordering::Relaxed);
        }
        KeyCode::KeyZ => controls.thin.store(0, Ordering::Relaxed),
        KeyCode::KeyX => controls.thin.store(1, Ordering::Relaxed),
        KeyCode::KeyC => controls.thin.store(2, Ordering::Relaxed),
        KeyCode::KeyV => controls.thin.store(3, Ordering::Relaxed),
        KeyCode::KeyG => {
            controls.shadow_classes.fetch_xor(1, Ordering::Relaxed);
        }
        KeyCode::KeyM => {
            controls.thin_tint.fetch_xor(1, Ordering::Relaxed);
        }
        KeyCode::KeyL => {
            let forced = controls.forced_lod.load(Ordering::Relaxed);
            let next = match forced {
                NO_FORCED_LOD => 0,
                7 => NO_FORCED_LOD,
                lod => lod + 1,
            };
            controls.forced_lod.store(next, Ordering::Relaxed);
        }
        KeyCode::Comma | KeyCode::Period => {
            let threshold = f32::from_bits(controls.threshold_bits.load(Ordering::Relaxed));
            let factor = if code == KeyCode::Comma { 0.5 } else { 2.0 };
            controls
                .threshold_bits
                .store((threshold * factor).to_bits(), Ordering::Relaxed);
        }
        _ => return,
    }
    let (control, threshold_pixels, band) = uniform_words();
    let forced_lod = (control >> 4) & 15;
    tracing::info!(
        mode = MODES[(control & 15) as usize],
        forced_lod = (forced_lod != NO_FORCED_LOD).then_some(forced_lod),
        threshold_pixels,
        band,
        tint = (control >> 17) & 1 == 1,
        thin = THIN_MODES[((control >> 18) & 3) as usize],
        shadow_classes = (control >> 20) & 1 == 1,
        thin_tint = (control >> 21) & 1 == 1,
        "PROTOTYPE controls"
    );
}

/// `praxis prototype-series <directory> <frame>[,<frame>...] [<variant>...]`:
/// renders `town` at 2560×1440 at each frame for the reference (every
/// Instance at LOD 0) and each variant, `<mode>[:<band>[:<threshold>]]`, and
/// prints how each variant's frames differ from the reference's, and how
/// that difference changes from each frame to the next. Writes each frame
/// and its difference from the reference, eight times over, unless
/// `PRAXIS_PROTOTYPE_STATS_ONLY` is set.
#[cfg(feature = "cook")]
pub fn series(arguments: &[String]) -> anyhow::Result<()> {
    use anyhow::Context as _;
    let [directory, frames, variants @ ..] = arguments else {
        anyhow::bail!("prototype-series <directory> <frames> [<variant>...]");
    };
    let directory = std::path::Path::new(directory);
    std::fs::create_dir_all(directory)?;
    let frames: Vec<u32> = frames
        .split(',')
        .map(|frame| frame.parse().context("a frame is a number"))
        .collect::<anyhow::Result<_>>()?;
    let (width, height) = (2560, 1440);
    let context = pollster::block_on(crate::gpu::GpuContext::headless())?;
    let write = std::env::var_os("PRAXIS_PROTOTYPE_STATS_ONLY").is_none();
    let save = |name: &str, frame: u32, pixels: &[u8]| -> anyhow::Result<()> {
        if write {
            image::save_buffer_with_format(
                directory.join(format!("{name}-frame-{frame}.png")),
                pixels,
                width,
                height,
                image::ExtendedColorType::Rgba8,
                image::ImageFormat::Png,
            )?;
        }
        Ok(())
    };
    let render = |mode: u32, band: f32, threshold: f32| {
        let controls = controls();
        controls.mode.store(mode, Ordering::Relaxed);
        controls.band_bits.store(band.to_bits(), Ordering::Relaxed);
        controls
            .threshold_bits
            .store(threshold.to_bits(), Ordering::Relaxed);
        crate::capture::capture_frames(
            &context,
            crate::ReferenceScene::Town,
            glam::DVec3::ZERO,
            width,
            height,
            &frames,
            crate::RenderSwitches::default(),
        )
    };
    // The reference: every Instance at LOD 0, or PRAXIS_PROTOTYPE_REFERENCE's
    // variant.
    let parse = |variant: &str| -> anyhow::Result<(u32, f32, f32)> {
        let mut parts = variant.split(':');
        let mode_name = parts.next().unwrap_or_default();
        let mode = MODES
            .iter()
            .position(|&mode| mode == mode_name)
            .with_context(|| format!("no mode {mode_name:?}"))? as u32;
        Ok((
            mode,
            parts.next().map_or(Ok(1.5), str::parse)?,
            parts.next().map_or(Ok(1.0), str::parse)?,
        ))
    };
    let reference_variant =
        std::env::var("PRAXIS_PROTOTYPE_REFERENCE").unwrap_or_else(|_| "off".to_owned());
    let (mode, band, threshold) = parse(&reference_variant)?;
    let reference = render(mode, band, threshold)?;
    for (&frame, pixels) in frames.iter().zip(&reference) {
        save(&reference_variant.replace(':', "-"), frame, pixels)?;
    }
    // How many pixels of a variant's own frames change by more than 8 and
    // by more than 32 from each frame to the next, with no reference.
    let own_changes = |name: &str, rendered: &[Vec<u8>]| {
        for (pair, &frame) in rendered.windows(2).zip(&frames[1..]) {
            let changes = pair[0].as_chunks::<4>().0.iter().zip(pair[1].as_chunks::<4>().0).map(
                |(before, after)| {
                    (0..3)
                        .map(|channel| before[channel].abs_diff(after[channel]))
                        .max()
                        .unwrap_or(0)
                },
            );
            println!(
                "own {name} {frame} {} {}",
                changes.clone().filter(|&change| change > 8).count(),
                changes.filter(|&change| change > 32).count()
            );
        }
    };
    own_changes(&reference_variant.replace(':', "-"), &reference);
    println!(
        "variant frame differing_pixels max_delta mean_delta_per_mille change_over_8 change_over_32 max_change"
    );
    for variant in variants {
        let (mode, band, threshold) = parse(variant)?;
        let rendered = render(mode, band, threshold)?;
        let name = variant.replace(':', "-");
        own_changes(&name, &rendered);
        let mut previous: Option<Vec<i16>> = None;
        for ((&frame, pixels), reference_pixels) in frames.iter().zip(&rendered).zip(&reference) {
            let deltas: Vec<i16> = pixels
                .iter()
                .zip(reference_pixels)
                .map(|(&channel, &reference_channel)| {
                    i16::from(channel) - i16::from(reference_channel)
                })
                .collect();
            let pixel_deltas = deltas.as_chunks::<4>().0.iter().map(|pixel| {
                pixel[..3]
                    .iter()
                    .map(|channel| channel.unsigned_abs())
                    .max()
                    .unwrap_or(0)
            });
            let differing = pixel_deltas.clone().filter(|&delta| delta > 1).count();
            let max_delta = pixel_deltas.clone().max().unwrap_or(0);
            let mean_per_mille = pixel_deltas.map(u64::from).sum::<u64>() as f64 * 1000.0
                / 255.0
                / f64::from(width * height);
            // How far each pixel's difference from the reference moved since
            // the frame before in the list: a pop where it jumps.
            let (over_8, over_32, max_change) = previous.as_ref().map_or((0, 0, 0), |before| {
                let changes = deltas
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .zip(before.as_chunks::<4>().0)
                    .map(|(now, then)| {
                        (0..3)
                            .map(|channel| (now[channel] - then[channel]).unsigned_abs())
                            .max()
                            .unwrap_or(0)
                    });
                (
                    changes.clone().filter(|&change| change > 8).count(),
                    changes.clone().filter(|&change| change > 32).count(),
                    changes.max().unwrap_or(0),
                )
            });
            println!(
                "{name} {frame} {differing} {max_delta} {mean_per_mille:.3} {over_8} {over_32} {max_change}"
            );
            save(&name, frame, pixels)?;
            let amplified: Vec<u8> = deltas
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|pixel| {
                    let amplify = |channel: i16| (channel.unsigned_abs() * 8).min(255) as u8;
                    [amplify(pixel[0]), amplify(pixel[1]), amplify(pixel[2]), 255]
                })
                .collect();
            save(
                &format!("{name}-difference-from-{}", reference_variant.replace(':', "-")),
                frame,
                &amplified,
            )?;
            previous = Some(deltas);
        }
    }
    Ok(())
}

/// PROTOTYPE (ticket 801). `praxis prototype-thin <directory>
/// <frame>[,<frame>...] [<variant>...]`: renders `town` at 2560×1440 at each
/// frame for each variant, a thin mode with `+classes` for the shadow
/// classes, beside a reference rendered as cooked at three times the size
/// and box-filtered in linear light. Writes each image and each variant's
/// difference from the reference, eight times over, and prints, over the
/// pixels the widened thin Meshes cover, each variant's mean error and
/// root-mean-square error in 8-bit luminance steps and how much that error
/// changes from each listed frame to the next.
#[cfg(feature = "cook")]
pub fn thin_series(arguments: &[String]) -> anyhow::Result<()> {
    use anyhow::Context as _;
    let [directory, frames, variants @ ..] = arguments else {
        anyhow::bail!("prototype-thin <directory> <frames> [<variant>...]");
    };
    anyhow::ensure!(thin_enabled(), "set PRAXIS_PROTOTYPE_THIN");
    let directory = std::path::Path::new(directory);
    std::fs::create_dir_all(directory)?;
    let frames: Vec<u32> = frames
        .split(',')
        .map(|frame| frame.parse().context("a frame is a number"))
        .collect::<anyhow::Result<_>>()?;
    let (width, height) = (2560u32, 1440u32);
    let context = pollster::block_on(crate::gpu::GpuContext::headless())?;
    let save = |name: &str, frame: u32, pixels: &[u8]| -> anyhow::Result<()> {
        image::save_buffer_with_format(
            directory.join(format!("{name}-frame-{frame}.png")),
            pixels,
            width,
            height,
            image::ExtendedColorType::Rgba8,
            image::ImageFormat::Png,
        )?;
        Ok(())
    };
    let render = |thin: u32, classes: bool, tint: bool, scale: u32, frames: &[u32]| {
        let controls = controls();
        controls.thin.store(thin, Ordering::Relaxed);
        controls
            .shadow_classes
            .store(u32::from(classes), Ordering::Relaxed);
        controls.thin_tint.store(u32::from(tint), Ordering::Relaxed);
        crate::capture::capture_frames(
            &context,
            crate::ReferenceScene::Town,
            glam::DVec3::ZERO,
            width * scale,
            height * scale,
            frames,
            crate::RenderSwitches::default(),
        )
    };
    let to_linear: Vec<f32> = (0..256)
        .map(|step| {
            let encoded = step as f32 / 255.0;
            if encoded <= 0.04045 {
                encoded / 12.92
            } else {
                ((encoded + 0.055) / 1.055).powf(2.4)
            }
        })
        .collect();
    let to_srgb = |linear: f32| {
        let encoded = if linear <= 0.003_130_8 {
            linear * 12.92
        } else {
            1.055 * linear.powf(1.0 / 2.4) - 0.055
        };
        (encoded * 255.0).round().clamp(0.0, 255.0) as u8
    };
    const SCALE: u32 = 3;
    let box_filtered = |large: &[u8]| -> Vec<u8> {
        let large_width = (width * SCALE) as usize;
        let mut small = Vec::with_capacity((width * height * 4) as usize);
        for row in 0..height as usize {
            for column in 0..width as usize {
                let mut sums = [0.0f32; 3];
                for down in 0..SCALE as usize {
                    for across in 0..SCALE as usize {
                        let texel = ((row * SCALE as usize + down) * large_width
                            + column * SCALE as usize
                            + across)
                            * 4;
                        for channel in 0..3 {
                            sums[channel] += to_linear[large[texel + channel] as usize];
                        }
                    }
                }
                let samples = (SCALE * SCALE) as f32;
                small.extend(sums.map(|sum| to_srgb(sum / samples)));
                small.push(255);
            }
        }
        small
    };
    // The reference, a few frames a render, as each is 133 MB.
    let mut reference = Vec::new();
    for batch in frames.chunks(4) {
        for large in render(0, false, false, SCALE, batch)? {
            reference.push(box_filtered(&large));
        }
    }
    for (&frame, pixels) in frames.iter().zip(&reference) {
        save("reference", frame, pixels)?;
    }
    // The pixels the widened thin Meshes cover: where tinting them shows.
    let widened = render(3, false, false, 1, &frames)?;
    let tinted = render(3, false, true, 1, &frames)?;
    let masks: Vec<Vec<bool>> = widened
        .iter()
        .zip(&tinted)
        .map(|(plain, tinted)| {
            plain
                .as_chunks::<4>()
                .0
                .iter()
                .zip(tinted.as_chunks::<4>().0)
                .map(|(plain, tinted)| plain != tinted)
                .collect()
        })
        .collect();
    let luminance = |pixel: &[u8; 4]| {
        0.2126 * f32::from(pixel[0]) + 0.7152 * f32::from(pixel[1]) + 0.0722 * f32::from(pixel[2])
    };
    println!("variant frame thin_pixels mean_error rms_error mean_error_change");
    for variant in variants {
        let (mode_name, classes) = match variant.strip_suffix("+classes") {
            Some(mode_name) => (mode_name, true),
            None => (variant.as_str(), false),
        };
        let thin = THIN_MODES
            .iter()
            .position(|&mode| mode == mode_name)
            .with_context(|| format!("no thin mode {mode_name:?}"))? as u32;
        let rendered = render(thin, classes, false, 1, &frames)?;
        let name = variant.replace('+', "-");
        let mut previous: Option<Vec<f32>> = None;
        for (((&frame, pixels), reference_pixels), mask) in
            frames.iter().zip(&rendered).zip(&reference).zip(&masks)
        {
            save(&name, frame, pixels)?;
            let errors: Vec<f32> = pixels
                .as_chunks::<4>()
                .0
                .iter()
                .zip(reference_pixels.as_chunks::<4>().0)
                .map(|(pixel, reference_pixel)| luminance(pixel) - luminance(reference_pixel))
                .collect();
            let thin_pixels = mask.iter().filter(|&&thin| thin).count().max(1) as f32;
            let masked = || errors.iter().zip(mask).filter(|&(_, &thin)| thin).map(|(error, _)| *error);
            let mean = masked().sum::<f32>() / thin_pixels;
            let rms = (masked().map(|error| error * error).sum::<f32>() / thin_pixels).sqrt();
            let change = previous.as_ref().map_or(0.0, |before| {
                errors
                    .iter()
                    .zip(before)
                    .zip(mask)
                    .filter(|&(_, &thin)| thin)
                    .map(|((now, then), _)| (now - then).abs())
                    .sum::<f32>()
                    / thin_pixels
            });
            println!("{name} {frame} {thin_pixels} {mean:.2} {rms:.2} {change:.2}");
            let amplified: Vec<u8> = pixels
                .as_chunks::<4>()
                .0
                .iter()
                .zip(reference_pixels.as_chunks::<4>().0)
                .flat_map(|(pixel, reference_pixel)| {
                    let amplify = |channel: usize| {
                        (u16::from(pixel[channel].abs_diff(reference_pixel[channel])) * 8).min(255)
                            as u8
                    };
                    [amplify(0), amplify(1), amplify(2), 255]
                })
                .collect();
            save(&format!("{name}-difference-from-reference"), frame, &amplified)?;
            previous = Some(errors);
        }
    }
    Ok(())
}
