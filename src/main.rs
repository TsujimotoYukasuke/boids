mod boids;
mod debug;

use std::f32::consts::PI;

use bevy::{input::mouse::MouseMotion, prelude::*};
use boids::BoidsPlugin;
use debug::DebugPlugin;

const CAMERA_SENSITIVITY: f32 = 0.1;
const CLEAR_COLOR: Color = Color::rgb(0.0, 0.05, 0.1);
const MOVEMENT_SPEED: f32 = 5.0;
const WINDOW_WIDTH: f32 = 1600.0;
const WINDOW_HEIGHT: f32 = 900.0;

#[derive(Default)]
struct CameraOrientation {
    yaw: f32,
    pitch: f32,
}

fn main() {
    App::new()
        .insert_resource(CameraOrientation::default())
        .insert_resource(ClearColor(CLEAR_COLOR))
        .insert_resource(WindowDescriptor {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            title: String::from("Boids"),
            present_mode: bevy::window::PresentMode::Fifo,
            resizable: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(BoidsPlugin)
        .add_plugin(DebugPlugin)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_light)
        .add_system(capture_cursor)
        .add_system(move_camera)
        .add_system(rotate_camera)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::Z, Vec3::Y),
            ..Default::default()
        })
        .insert(Name::new("Player Camera"));
}

fn spawn_light(mut commands: Commands) {
    commands
        .spawn_bundle(DirectionalLightBundle::default())
        .insert(Name::new("Sun Light"));
}

fn capture_cursor(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut windows: ResMut<Windows>,
) {
    let window = windows.get_primary_mut().unwrap();

    if mouse_input.just_pressed(MouseButton::Left) {
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        window.set_cursor_lock_mode(false);
        window.set_cursor_visibility(true);
    }
}

fn move_camera(
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
) {
    let mut camera_transform = camera_query.get_single_mut().unwrap();

    let mut horizontal_input = Vec2::new(0.0, 0.0);
    if keyboard_input.pressed(KeyCode::W) {
        horizontal_input.y += MOVEMENT_SPEED * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::S) {
        horizontal_input.y -= MOVEMENT_SPEED * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::A) {
        horizontal_input.x -= MOVEMENT_SPEED * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::D) {
        horizontal_input.x += MOVEMENT_SPEED * time.delta_seconds();
    }

    let mut vertical_input = 0.0;
    if keyboard_input.pressed(KeyCode::E) {
        vertical_input += MOVEMENT_SPEED * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::Q) {
        vertical_input -= MOVEMENT_SPEED * time.delta_seconds();
    }

    let forward_movement = camera_transform.forward() * horizontal_input.y;
    let strafe_movement = camera_transform.right() * horizontal_input.x;

    camera_transform.translation +=
        forward_movement + strafe_movement + Vec3::new(0.0, vertical_input, 0.0);
}

fn rotate_camera(
    mut mouse_movement: EventReader<MouseMotion>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    mut camera_orientation: ResMut<CameraOrientation>,
    time: Res<Time>,
    windows: Res<Windows>,
) {
    let mut camera_transform = camera_query.get_single_mut().unwrap();

    let cursor_captured = match windows.get_primary() {
        Some(window) => window.cursor_locked(),
        None => false,
    };

    if cursor_captured {
        for mouse_movement in mouse_movement.iter() {
            camera_orientation.yaw -=
                mouse_movement.delta.x * time.delta_seconds() * CAMERA_SENSITIVITY;
            camera_orientation.pitch = f32::clamp(
                camera_orientation.pitch
                    - mouse_movement.delta.y * time.delta_seconds() * CAMERA_SENSITIVITY,
                -0.5 * PI,
                0.5 * PI,
            );
        }
    }

    let rotation_yaw = Quat::from_rotation_y(camera_orientation.yaw);
    let right_axis = rotation_yaw * Vec3::X;
    let rotation_pitch = Quat::from_axis_angle(right_axis, camera_orientation.pitch);

    camera_transform.rotation = rotation_pitch * rotation_yaw;
}
