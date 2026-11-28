use std::io::Write;

use glam::{DVec3, dvec3};
use praxis::gpu::GpuContext;
use praxis::{ReferenceScene, RenderSwitches};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;
const FIELD_OF_VIEW_RADIANS: f64 = std::f64::consts::FRAC_PI_4;

struct View {
    frame: u32,
    camera: DVec3,
    yaw_degrees: f64,
    pitch_degrees: f64,
}

const ROOM_A: View = View {
    frame: 690,
    camera: DVec3::new(-1.6, 1.65, -1.3),
    yaw_degrees: -255.0,
    pitch_degrees: -17.0,
};
const ROOM_B: View = View {
    frame: 1260,
    camera: DVec3::new(0.7, 1.65, -1.6),
    yaw_degrees: -296.0,
    pitch_degrees: -19.0,
};

fn pixel_seeing(view: &View, point: DVec3) -> (u32, u32) {
    let (yaw, pitch) = (
        view.yaw_degrees.to_radians(),
        view.pitch_degrees.to_radians(),
    );
    let forward = dvec3(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    );
    let right = forward.cross(DVec3::Y).normalize();
    let up = right.cross(forward);
    let to_point = point - view.camera;
    let depth = to_point.dot(forward);
    let half_height = (FIELD_OF_VIEW_RADIANS / 2.0).tan();
    let half_width = half_height * f64::from(WIDTH) / f64::from(HEIGHT);
    let across = to_point.dot(right) / (depth * half_width);
    let along_up = to_point.dot(up) / (depth * half_height);
    (
        ((across + 1.0) / 2.0 * f64::from(WIDTH)) as u32,
        ((1.0 - along_up) / 2.0 * f64::from(HEIGHT)) as u32,
    )
}

fn mean_channel_around(image: &image::RgbaImage, view: &View, point: DVec3) -> f64 {
    let (centre_x, centre_y) = pixel_seeing(view, point);
    let window = (centre_x - 2..=centre_x + 2)
        .flat_map(|x| (centre_y - 2..=centre_y + 2).map(move |y| image.get_pixel(x, y)));
    let sum: u32 = window
        .flat_map(|pixel| &pixel.0[..3])
        .map(|&channel| u32::from(channel))
        .sum();
    f64::from(sum) / 75.0
}

#[test]
fn the_sun_patch_and_the_wall_by_the_lamp_are_lit_and_the_floor_and_wall_beyond_them_are_not() {
    let context = match pollster::block_on(GpuContext::headless()) {
        Ok(context) => context,
        Err(error) if error.is::<wgpu::RequestAdapterError>() => {
            // Not `eprintln!`, which libtest captures from passing tests.
            writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
            return;
        }
        Err(error) => panic!("headless GPU context failed: {error:#}"),
    };
    let mut frames = praxis::capture_frames(
        &context,
        ReferenceScene::House,
        DVec3::ZERO,
        WIDTH,
        HEIGHT,
        &[ROOM_A.frame, ROOM_B.frame],
        RenderSwitches::default(),
    )
    .expect("capture failed")
    .into_iter()
    .map(|pixels| image::RgbaImage::from_raw(WIDTH, HEIGHT, pixels).expect("WIDTH × HEIGHT"));
    let (room_a, room_b) = (frames.next().unwrap(), frames.next().unwrap());

    let sun_patch = mean_channel_around(&room_b, &ROOM_B, dvec3(1.7, 0.05, 0.55));
    let floor_beside = mean_channel_around(&room_b, &ROOM_B, dvec3(2.9, 0.05, 0.3));
    assert!(
        sun_patch > floor_beside + 60.0,
        "the sun patch reads {sun_patch}, the floor beside it {floor_beside}"
    );

    let wall_by_the_lamp = mean_channel_around(&room_a, &ROOM_A, dvec3(-3.3, 1.6, 2.0));
    let wall_no_light_reaches = mean_channel_around(&room_b, &ROOM_B, dvec3(4.05, 1.4, 1.2));
    assert!(
        wall_by_the_lamp > wall_no_light_reaches + 20.0,
        "room A's wall by the lamp reads {wall_by_the_lamp}, room B's east wall {wall_no_light_reaches}"
    );
}
