//! Shadow rays: on the `shadows` Reference scene, tracing the sun's shadow
//! rays darkens exactly the pixels where the sphere's analytic shadow falls
//! on the ground, and changes no other, outside a one-pixel band around every
//! edge of what each pixel sees. The oracle is the scene's geometry as
//! `src/scenes/mod.rs` places it, intersected in f64: the true sphere, not
//! its triangles, which lie within 2.1 mm of it, so their shadow's outline
//! lies within 2.5 mm of the true one's, a quarter of a pixel there. Near the
//! sphere's terminator, though, a triangle can face away from the sun while
//! the normals interpolated over it face the sun: there it hides the sun from
//! its own pixels, which are not judged.

use std::io::Write;
use std::path::Path;

use praxis::gpu::GpuContext;
use praxis::{ReferenceScene, RenderSwitches};

/// The capture command's default size, as the Goldens'.
const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;

/// The sphere: its centre 1 m over the ground plane, which lies at height 0
/// and reaches 8 m from the origin along X and Z.
const SPHERE_CENTRE: glam::DVec3 = glam::DVec3::new(0.0, 1.0, 0.0);
const SPHERE_RADIUS: f64 = 0.5;
const GROUND_HALF_SIZE: f64 = 8.0;

/// The camera and what it looks at, through a 45° vertical field of view.
const CAMERA: glam::DVec3 = glam::DVec3::new(0.0, 2.4, 3.0);
const CAMERA_TARGET: glam::DVec3 = glam::DVec3::new(0.0, 0.45, -0.4);
const FIELD_OF_VIEW: f64 = std::f64::consts::FRAC_PI_4;

/// Where the sun shines along.
const SUN_DIRECTION: glam::DVec3 = glam::DVec3::new(0.5, -1.0, -0.3);

/// The cosine of the sun's angle from the sphere's normal past which a
/// triangle can face away from the sun: the sphere's 48 × 24 segments of
/// 7.5° make triangles whose circumradius, √2 sin 3.75° of the radius, turns
/// them up to 5.3° from the normals interpolated over them.
const TERMINATOR_BAND: f64 = std::f64::consts::SQRT_2 * 0.065_403_129_230_143_07;

/// The largest channel difference, in 8-bit steps, that a pixel the rays do
/// not shadow may show, as the Goldens allow.
const TOLERANCE: i16 = 1;

/// What the centre of a pixel sees.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Sight {
    Sky,
    /// The sphere where the sun reaches it, beyond the terminator's band.
    SunlitSphere,
    /// The sphere within `TERMINATOR_BAND` of its terminator, on its sunlit
    /// side.
    Terminator,
    /// The sphere where it faces away from the sun.
    UnlitSphere,
    SunlitGround,
    /// The ground where the sphere hides the sun.
    ShadowedGround,
}

/// The distance along unit `direction` from `origin` to where it enters the
/// sphere, if it does ahead of `origin`.
fn enters_sphere(origin: glam::DVec3, direction: glam::DVec3) -> Option<f64> {
    // Where |origin + t direction − centre|² = radius², a quadratic in t.
    let to_origin = origin - SPHERE_CENTRE;
    let along = to_origin.dot(direction);
    let discriminant = along * along - (to_origin.length_squared() - SPHERE_RADIUS.powi(2));
    let distance = -along - discriminant.sqrt();
    (discriminant > 0.0 && distance > 0.0).then_some(distance)
}

/// What the camera sees through `(x, y)`, in pixels from the image's top
/// left corner.
fn sight(x: f64, y: f64) -> Sight {
    let forward = (CAMERA_TARGET - CAMERA).normalize();
    let right = forward.cross(glam::DVec3::Y).normalize();
    let up = right.cross(forward);
    let half_height = (FIELD_OF_VIEW / 2.0).tan();
    let half_width = half_height * f64::from(WIDTH) / f64::from(HEIGHT);
    let across = (2.0 * x / f64::from(WIDTH) - 1.0) * half_width;
    let along_up = (1.0 - 2.0 * y / f64::from(HEIGHT)) * half_height;
    let direction = (forward + right * across + up * along_up).normalize();
    let towards_sun = -SUN_DIRECTION.normalize();

    let to_ground = (direction.y < 0.0).then(|| -CAMERA.y / direction.y);
    let ground = to_ground
        .map(|distance| CAMERA + direction * distance)
        .filter(|point| point.x.abs() <= GROUND_HALF_SIZE && point.z.abs() <= GROUND_HALF_SIZE);
    if let Some(distance) = enters_sphere(CAMERA, direction)
        && ground.is_none_or(|point| distance < (point - CAMERA).length())
    {
        let normal = (CAMERA + direction * distance - SPHERE_CENTRE) / SPHERE_RADIUS;
        return match normal.dot(towards_sun) {
            cosine if cosine > TERMINATOR_BAND => Sight::SunlitSphere,
            cosine if cosine > 0.0 => Sight::Terminator,
            _ => Sight::UnlitSphere,
        };
    }
    match ground {
        None => Sight::Sky,
        Some(point) if enters_sphere(point, towards_sun).is_some() => Sight::ShadowedGround,
        Some(_) => Sight::SunlitGround,
    }
}

/// The last frame `praxis::capture` renders of the scene, as `switches` set.
fn capture(context: &GpuContext, switches: RenderSwitches) -> image::RgbaImage {
    let (pixels, _) = praxis::capture(
        context,
        ReferenceScene::Shadows,
        glam::DVec3::ZERO,
        WIDTH,
        HEIGHT,
        switches,
    )
    .expect("capture failed");
    image::RgbaImage::from_raw(WIDTH, HEIGHT, pixels).expect("WIDTH × HEIGHT pixels")
}

#[test]
fn the_suns_shadow_rays_darken_exactly_the_spheres_analytic_shadow() {
    let context = match pollster::block_on(GpuContext::headless()) {
        Ok(context) => context,
        Err(error) if error.is::<wgpu::RequestAdapterError>() => {
            // Not `eprintln!`, which libtest captures from passing tests.
            writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
            return;
        }
        Err(error) => panic!("headless GPU context failed: {error:#}"),
    };
    let traced = capture(&context, RenderSwitches::default());
    let untraced = capture(
        &context,
        RenderSwitches {
            shadow_rays: false,
            ..RenderSwitches::default()
        },
    );

    // Each pixel whose centre and its eight neighbours' centres see alike
    // is judged: where it sees shadowed ground, darkened by more than the
    // tolerance in some channel and brightened in none; elsewhere, unchanged.
    let sights: Vec<Sight> = (0..HEIGHT)
        .flat_map(|y| (0..WIDTH).map(move |x| sight(f64::from(x) + 0.5, f64::from(y) + 0.5)))
        .collect();
    let sight_at = |x: i64, y: i64| {
        let (x, y) = (
            x.clamp(0, i64::from(WIDTH) - 1),
            y.clamp(0, i64::from(HEIGHT) - 1),
        );
        sights[(y * i64::from(WIDTH) + x) as usize]
    };
    let mut wrong = Vec::new();
    let mut judged_shadowed = 0;
    let mut diagnostic = traced.clone();
    for (x, y, traced_pixel) in traced.enumerate_pixels() {
        let (column, row) = (i64::from(x), i64::from(y));
        let expected = sight_at(column, row);
        let in_band = (-1..=1)
            .flat_map(|row_step| (-1..=1).map(move |column_step| (column_step, row_step)))
            .any(|(column_step, row_step)| {
                sight_at(column + column_step, row + row_step) != expected
            });
        if in_band || expected == Sight::Terminator {
            continue;
        }
        let untraced_pixel = untraced.get_pixel(x, y);
        let deltas = [0, 1, 2]
            .map(|channel| i16::from(traced_pixel[channel]) - i16::from(untraced_pixel[channel]));
        let shadowed = expected == Sight::ShadowedGround;
        let right = if shadowed {
            deltas.iter().any(|&delta| delta < -TOLERANCE) && deltas.iter().all(|&delta| delta <= 0)
        } else {
            deltas.iter().all(|&delta| delta.abs() <= TOLERANCE)
        };
        judged_shadowed += usize::from(shadowed);
        if !right {
            wrong.push((x, y, expected, deltas));
            diagnostic.put_pixel(x, y, image::Rgba([255, 0, 255, 255]));
        }
    }

    // The shadow is thousands of pixels, so the check has something to judge.
    assert!(
        judged_shadowed > 4000,
        "{judged_shadowed} shadowed pixels judged"
    );
    if !wrong.is_empty() {
        let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join("shadows");
        std::fs::create_dir_all(&directory).expect("diagnostic directory");
        let path = directory.join("wrong-pixels.png");
        diagnostic.save(&path).expect("write diagnostic image");
        panic!(
            "{} of the pixels judged are wrong, in magenta in {}; the first: {:?}",
            wrong.len(),
            path.display(),
            &wrong[..wrong.len().min(8)]
        );
    }
}
