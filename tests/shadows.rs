//! Shadow rays: on the `shadows` Reference scene, tracing each light's shadow
//! rays darkens exactly the pixels where the sphere's analytic shadows fall
//! on the ground, the sun's and the point light's, and changes no other,
//! outside a one-pixel band around every edge of what each pixel sees. The
//! oracle is the scene's geometry and lights as `src/scenes/mod.rs` places
//! them, intersected in f64: the true sphere, not its triangles, which lie
//! within 2.1 mm of it, so their shadows' outlines lie within 2.5 mm of the
//! true ones' under the sun, a quarter of a pixel there, and within 11 mm
//! under the point light, whose rays graze the ground farther out, three
//! quarters of a pixel there. Near the sphere's terminator for each light,
//! though, a triangle can face away from the light while the normals
//! interpolated over it face it: there it hides the light from its own
//! pixels, which are not judged.

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

/// A light of the scene, as far as its shadow goes.
#[derive(Clone, Copy)]
enum Light {
    /// Shining along a direction.
    Directional(glam::DVec3),
    /// At a position, whose range of 20 m reaches every point of the ground
    /// from where the scene places it: the farthest corner is 13.3 m away.
    Point(glam::DVec3),
}

/// The sun, then the point light.
const LIGHTS: [Light; 2] = [
    Light::Directional(glam::DVec3::new(0.5, -1.0, -0.3)),
    Light::Point(glam::DVec3::new(1.5, 2.5, 1.0)),
];

impl Light {
    /// The unit direction from `point` toward the light, and how far a
    /// shadow ray runs that way to reach it.
    fn towards(self, point: glam::DVec3) -> (glam::DVec3, f64) {
        match self {
            Self::Directional(direction) => (-direction.normalize(), f64::INFINITY),
            Self::Point(position) => {
                let offset = position - point;
                (offset.normalize(), offset.length())
            }
        }
    }
}

/// The cosine of a light's angle from the sphere's normal past which a
/// triangle can face away from the light: the sphere's 48 × 24 segments of
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
    /// The sphere beyond each light's terminator band, and which of the
    /// lights, in `LIGHTS`'s order, reach it.
    Sphere {
        lit: [bool; LIGHTS.len()],
    },
    /// The sphere within `TERMINATOR_BAND` of a light's terminator, on its
    /// lit side.
    Terminator,
    /// The ground, and which of the lights the sphere hides from it.
    Ground {
        shadowed: [bool; LIGHTS.len()],
    },
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

    let to_ground = (direction.y < 0.0).then(|| -CAMERA.y / direction.y);
    let ground = to_ground
        .map(|distance| CAMERA + direction * distance)
        .filter(|point| point.x.abs() <= GROUND_HALF_SIZE && point.z.abs() <= GROUND_HALF_SIZE);
    if let Some(distance) = enters_sphere(CAMERA, direction)
        && ground.is_none_or(|point| distance < (point - CAMERA).length())
    {
        let point = CAMERA + direction * distance;
        let normal = (point - SPHERE_CENTRE) / SPHERE_RADIUS;
        let cosines = LIGHTS.map(|light| normal.dot(light.towards(point).0));
        if cosines
            .iter()
            .any(|&cosine| cosine > 0.0 && cosine <= TERMINATOR_BAND)
        {
            return Sight::Terminator;
        }
        return Sight::Sphere {
            lit: cosines.map(|cosine| cosine > 0.0),
        };
    }
    match ground {
        None => Sight::Sky,
        Some(point) => Sight::Ground {
            shadowed: LIGHTS.map(|light| {
                let (towards, reach) = light.towards(point);
                enters_sphere(point, towards).is_some_and(|distance| distance < reach)
            }),
        },
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
fn shadow_rays_darken_exactly_the_spheres_analytic_shadows() {
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
    // is judged: where it sees ground the sphere hides from a light,
    // darkened by more than the tolerance in some channel and brightened in
    // none; elsewhere, unchanged.
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
    let mut judged_shadowed = [0; LIGHTS.len()];
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
        let shadowed = match expected {
            Sight::Ground { shadowed } => shadowed,
            _ => [false; LIGHTS.len()],
        };
        let right = if shadowed.contains(&true) {
            deltas.iter().any(|&delta| delta < -TOLERANCE) && deltas.iter().all(|&delta| delta <= 0)
        } else {
            deltas.iter().all(|&delta| delta.abs() <= TOLERANCE)
        };
        for (count, shadowed) in judged_shadowed.iter_mut().zip(shadowed) {
            *count += usize::from(shadowed);
        }
        if !right {
            wrong.push((x, y, expected, deltas));
            diagnostic.put_pixel(x, y, image::Rgba([255, 0, 255, 255]));
        }
    }

    // Each light's shadow is thousands of pixels, so the check has something
    // to judge under both.
    assert!(
        judged_shadowed.iter().all(|&count| count > 4000),
        "{judged_shadowed:?} shadowed pixels judged, the sun's then the point light's"
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
