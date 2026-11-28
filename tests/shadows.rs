//! Shadow rays: on the `shadows` Reference scene, tracing each light's shadow
//! rays darkens exactly the pixels where the sphere's analytic shadows fall
//! on the ground, the sun's and the point light's, and changes no other,
//! outside a one-pixel band around every edge of what each pixel sees. The
//! oracle is the scene's geometry and lights as `src/scenes/mod.rs` places
//! them, intersected in f64: the true sphere, not its triangles, which lie
//! within 2.1 mm of it, so their shadows' outlines lie within 2.5 mm of the
//! true ones' under the sun, a quarter of a pixel there, and within 11 mm
//! under the point light, whose rays graze the ground farther out, three
//! quarters of a pixel there. Nothing shadows the sphere, so the rays leave
//! each of its pixels unchanged, those up to each light's terminator
//! included, where a triangle can face away from the light while the normals
//! interpolated over it face it.
//!
//! The same oracle judges the scene's Buffer views.

use std::io::Write;
use std::path::Path;

use praxis::gpu::GpuContext;
use praxis::{BufferView, ReferenceScene, RenderSwitches};

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

/// The largest channel difference, in 8-bit steps, that a pixel the rays do
/// not shadow may show, as the Goldens allow.
const TOLERANCE: i16 = 1;

/// What the centre of a pixel sees.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Sight {
    Sky,
    Sphere,
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

/// The unit direction the camera looks along through `(x, y)`, in pixels
/// from the image's top left corner.
fn view_ray(x: f64, y: f64) -> glam::DVec3 {
    let forward = (CAMERA_TARGET - CAMERA).normalize();
    let right = forward.cross(glam::DVec3::Y).normalize();
    let up = right.cross(forward);
    let half_height = (FIELD_OF_VIEW / 2.0).tan();
    let half_width = half_height * f64::from(WIDTH) / f64::from(HEIGHT);
    let across = (2.0 * x / f64::from(WIDTH) - 1.0) * half_width;
    let along_up = (1.0 - 2.0 * y / f64::from(HEIGHT)) * half_height;
    (forward + right * across + up * along_up).normalize()
}

/// What the camera sees through `(x, y)`, in pixels from the image's top
/// left corner.
fn sight(x: f64, y: f64) -> Sight {
    let direction = view_ray(x, y);
    let to_ground = (direction.y < 0.0).then(|| -CAMERA.y / direction.y);
    let ground = to_ground
        .map(|distance| CAMERA + direction * distance)
        .filter(|point| point.x.abs() <= GROUND_HALF_SIZE && point.z.abs() <= GROUND_HALF_SIZE);
    if let Some(distance) = enters_sphere(CAMERA, direction)
        && ground.is_none_or(|point| distance < (point - CAMERA).length())
    {
        return Sight::Sphere;
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

/// What each pixel sees, rows top first, where its centre and those of the
/// pixels up to `margin` from it see alike; `None` in the band around every
/// edge of what the pixels see, which no test judges.
fn judged_sights(margin: i64) -> Vec<Option<Sight>> {
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
    (0..i64::from(HEIGHT))
        .flat_map(|row| (0..i64::from(WIDTH)).map(move |column| (column, row)))
        .map(|(column, row)| {
            let expected = sight_at(column, row);
            (-margin..=margin)
                .flat_map(|row_step| {
                    (-margin..=margin).map(move |column_step| (column_step, row_step))
                })
                .all(|(column_step, row_step)| {
                    sight_at(column + column_step, row + row_step) == expected
                })
                .then_some(expected)
        })
        .collect()
}

/// A headless GPU context; `None`, with a line saying so, on a machine
/// without an adapter.
fn gpu_context() -> Option<GpuContext> {
    match pollster::block_on(GpuContext::headless()) {
        Ok(context) => Some(context),
        Err(error) if error.is::<wgpu::RequestAdapterError>() => {
            // Not `eprintln!`, which libtest captures from passing tests.
            writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
            None
        }
        Err(error) => panic!("headless GPU context failed: {error:#}"),
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
    let Some(context) = gpu_context() else {
        return;
    };
    let traced = capture(&context, RenderSwitches::default());
    let untraced = capture(
        &context,
        RenderSwitches {
            shadow_rays: false,
            ..RenderSwitches::default()
        },
    );

    // Each judged pixel: where it sees ground the sphere hides from a light,
    // darkened by more than the tolerance in some channel and brightened in
    // none; elsewhere, unchanged.
    let sights = judged_sights(1);
    let mut wrong = Vec::new();
    let mut judged_shadowed = [0; LIGHTS.len()];
    let mut diagnostic = traced.clone();
    for (x, y, traced_pixel) in traced.enumerate_pixels() {
        let Some(expected) = sights[(y * WIDTH + x) as usize] else {
            continue;
        };
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

fn capture_view(context: &GpuContext, view: BufferView) -> image::RgbaImage {
    let buffer_view = Some(view);
    capture(
        context,
        RenderSwitches {
            buffer_view,
            ..RenderSwitches::default()
        },
    )
}

/// In each column, the ground above the sphere lies farther along the view
/// axis than any of the sphere.
#[test]
fn the_depth_view_is_brighter_on_the_sphere_than_on_the_ground_behind_it() {
    let Some(context) = gpu_context() else {
        return;
    };
    let view = capture_view(&context, BufferView::Depth);
    let sights = judged_sights(1);
    let forward = (CAMERA_TARGET - CAMERA).normalize();
    let mut sphere_pixels = 0;
    for x in 0..WIDTH {
        let mut brightest_ground_above = 0;
        for y in 0..HEIGHT {
            let image::Rgba([red, green, blue, _]) = *view.get_pixel(x, y);
            assert!(red == green && green == blue, "({x}, {y}) is not grey");
            match sights[(y * WIDTH + x) as usize] {
                Some(Sight::Sky) => assert_eq!(red, 0, "the sky at ({x}, {y})"),
                Some(Sight::Ground { .. }) => {
                    brightest_ground_above = brightest_ground_above.max(red);
                    let direction = view_ray(f64::from(x) + 0.5, f64::from(y) + 0.5);
                    let view_depth = -CAMERA.y / direction.y * direction.dot(forward);
                    let grey = 255.0 / (1.0 + view_depth / 16.0);
                    assert!((f64::from(red) - grey).abs() <= 1.0, "({x}, {y}): {red}");
                }
                Some(Sight::Sphere) => {
                    sphere_pixels += 1;
                    assert!(red > brightest_ground_above, "({x}, {y}): {red}");
                }
                None => {}
            }
        }
    }
    assert!(sphere_pixels > 4000, "{sphere_pixels} sphere pixels judged");
}

/// A level 0 texel holds the farthest depth of two by two pixels' samples,
/// and a level past the coarsest shows the coarsest: the sky's depth.
#[test]
fn the_depth_pyramid_view_is_the_depth_views_grey_or_just_darker() {
    let Some(context) = gpu_context() else {
        return;
    };
    let depth = capture_view(&context, BufferView::Depth);
    let pyramid = capture_view(&context, BufferView::DepthPyramid(0));
    for ((nearest, farthest), sight) in depth.pixels().zip(pyramid.pixels()).zip(judged_sights(2)) {
        let (nearest, farthest) = (nearest.0[0], farthest.0[0]);
        match sight {
            Some(Sight::Sky) => assert_eq!(farthest, 0),
            Some(_) => assert!(farthest <= nearest && nearest - farthest <= 2, "{farthest}"),
            None => {}
        }
    }
    let coarsest = capture_view(&context, BufferView::DepthPyramid(99));
    assert!(coarsest.pixels().all(|texel| texel.0 == [0, 0, 0, 255]));
}

#[test]
fn the_normals_views_of_the_ground_are_the_colour_of_straight_up() {
    let Some(context) = gpu_context() else {
        return;
    };
    for normals in [BufferView::SurfaceNormals, BufferView::TriangleNormals] {
        let view = capture_view(&context, normals);
        for (sight, pixel) in judged_sights(1).into_iter().zip(view.pixels()) {
            if matches!(sight, Some(Sight::Ground { .. })) {
                let straight_up = (pixel.0.iter().zip([128, 255, 128]))
                    .all(|(&channel, up): (&u8, u8)| channel.abs_diff(up) <= 1);
                assert!(straight_up, "{normals:?} shows the ground as {pixel:?}");
            }
        }
    }
}

/// The scene's one point light reaches all the camera sees.
#[test]
fn the_point_lights_view_is_one_lights_heat_on_every_surface() {
    let Some(context) = gpu_context() else {
        return;
    };
    let view = capture_view(&context, BufferView::PointLights);
    for (sight, pixel) in judged_sights(1).into_iter().zip(view.pixels()) {
        match sight {
            Some(Sight::Sky) => assert_eq!(pixel.0, [0, 0, 0, 255]),
            Some(_) => assert!(pixel.0[0].abs_diff(96) <= 1 && pixel.0[1..] == [0, 0, 255]),
            None => {}
        }
    }
}

#[test]
fn the_shadow_mask_view_is_hidden_exactly_where_the_analytic_shadow_lies() {
    let Some(context) = gpu_context() else {
        return;
    };
    let (hidden, lit) = ([64, 64, 64, 255], [192, 192, 192, 255]);
    for light in 0..LIGHTS.len() {
        let view = capture_view(&context, BufferView::ShadowMask(light as u32));
        let mut hidden_pixels = 0;
        for (sight, pixel) in judged_sights(1).into_iter().zip(view.pixels()) {
            if let Some(Sight::Ground { shadowed }) = sight {
                hidden_pixels += usize::from(shadowed[light]);
                let expected = if shadowed[light] { hidden } else { lit };
                assert_eq!(pixel.0, expected, "light {light}, shadows {shadowed:?}");
            }
        }
        assert!(hidden_pixels > 4000, "{hidden_pixels} hidden pixels judged");
    }
}

/// The exposed colour of the ground at `point` under the sun alone, each
/// channel alike: glTF 2.0's dielectric BRDF
/// [ref:gltf-2.0-metal-and-dielectric-brdf], with its specular lobe
/// [ref:gltf-2.0-specular-brdf] and Fresnel term [ref:gltf-2.0-fresnel], of
/// the sun's light, and a tenth of the sun's illuminance as ambient light
/// reflected diffusely, at the exposure that takes the sun's illuminance to 1.
fn sunlit_ground(point: glam::DVec3) -> f64 {
    let (base_color, roughness, ambient_share) = (0.8, 0.6_f64, 0.1);
    let (to_light, _) = LIGHTS[0].towards(point);
    let to_camera = (CAMERA - point).normalize();
    let half_vector = (to_light + to_camera).normalize();
    let (normal_dot_light, normal_dot_view) = (to_light.y, to_camera.y);
    let alpha_squared = roughness.powi(4);
    let fresnel = 0.04 + 0.96 * (1.0 - to_camera.dot(half_vector)).powi(5);
    let distribution = alpha_squared
        / (std::f64::consts::PI * (half_vector.y.powi(2) * (alpha_squared - 1.0) + 1.0).powi(2));
    let shadowing =
        normal_dot_view * (alpha_squared + (1.0 - alpha_squared) * normal_dot_light.powi(2)).sqrt();
    let masking =
        normal_dot_light * (alpha_squared + (1.0 - alpha_squared) * normal_dot_view.powi(2)).sqrt();
    let diffuse = base_color / std::f64::consts::PI;
    let reflected =
        (1.0 - fresnel) * diffuse + fresnel * distribution / (2.0 * (shadowing + masking));
    reflected * normal_dot_light + ambient_share * diffuse
}

#[test]
fn the_hdr_view_of_the_sunlit_ground_is_its_luminance_times_the_exposure() {
    let Some(context) = gpu_context() else {
        return;
    };
    let view = capture_view(&context, BufferView::Hdr);
    let sun_alone = Some(Sight::Ground {
        shadowed: [false, true],
    });
    let mut sunlit_pixels = 0;
    for ((x, y, pixel), sight) in view.enumerate_pixels().zip(judged_sights(1)) {
        if sight != sun_alone {
            continue;
        }
        sunlit_pixels += 1;
        let direction = view_ray(f64::from(x) + 0.5, f64::from(y) + 0.5);
        let exposed = sunlit_ground(CAMERA - CAMERA.y / direction.y * direction);
        assert!(
            (0.0031308..1.0).contains(&exposed),
            "{exposed} at ({x}, {y})"
        );
        // The sRGB encoding above its linear segment [ref:khronos-dfs-srgb].
        let code = 255.0 * (1.055 * exposed.powf(1.0 / 2.4) - 0.055);
        let within_a_step = |&channel: &u8| (f64::from(channel) - code).abs() <= 1.0;
        assert!(
            pixel.0[..3].iter().all(within_a_step),
            "({x}, {y}): {pixel:?}, not {code}"
        );
    }
    assert!(sunlit_pixels > 4000, "{sunlit_pixels} sunlit pixels judged");
}
