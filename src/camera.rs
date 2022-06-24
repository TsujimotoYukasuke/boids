use std::f32::consts::PI;

use bevy::{input::mouse::MouseMotion, prelude::*};

#[derive(Component)]
struct CameraSettings {
    yaw: f32,
    pitch: f32,
    speed: f32,
    sensitivity: f32,
    fast_movement_multiplier: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            speed: 20.0,
            sensitivity: 0.1,
            fast_movement_multiplier: 2.5,
        }
    }
}

pub struct PlayerCameraPlugin;

impl Plugin for PlayerCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_camera)
            .add_system(capture_cursor)
            .add_system(move_camera)
            .add_system(rotate_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::Z, Vec3::Y),
            ..Default::default()
        })
        .insert(Name::new("Player Camera"))
        .insert(CameraSettings::default());
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
    mut camera_query: Query<(&CameraSettings, &mut Transform)>,
    time: Res<Time>,
) {
    let (camera_settings, mut camera_transform) = camera_query.get_single_mut().unwrap();
    let fast_movement = keyboard_input.pressed(KeyCode::LShift);

    let mut horizontal_input = Vec2::new(0.0, 0.0);
    if keyboard_input.pressed(KeyCode::W) {
        horizontal_input.y += match fast_movement {
            true => camera_settings.speed * camera_settings.fast_movement_multiplier,
            false => camera_settings.speed,
        } * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::S) {
        horizontal_input.y -= match fast_movement {
            true => camera_settings.speed * camera_settings.fast_movement_multiplier,
            false => camera_settings.speed,
        } * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::A) {
        horizontal_input.x -= match fast_movement {
            true => camera_settings.speed * camera_settings.fast_movement_multiplier,
            false => camera_settings.speed,
        } * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::D) {
        horizontal_input.x += match fast_movement {
            true => camera_settings.speed * camera_settings.fast_movement_multiplier,
            false => camera_settings.speed,
        } * time.delta_seconds();
    }

    let mut vertical_input = 0.0;
    if keyboard_input.pressed(KeyCode::E) {
        vertical_input += match fast_movement {
            true => camera_settings.speed * camera_settings.fast_movement_multiplier,
            false => camera_settings.speed,
        } * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::Q) {
        vertical_input -= match fast_movement {
            true => camera_settings.speed * camera_settings.fast_movement_multiplier,
            false => camera_settings.speed,
        } * time.delta_seconds();
    }

    let forward_movement = camera_transform.forward() * horizontal_input.y;
    let strafe_movement = camera_transform.right() * horizontal_input.x;

    camera_transform.translation +=
        forward_movement + strafe_movement + Vec3::new(0.0, vertical_input, 0.0);
}

fn rotate_camera(
    mut mouse_movement: EventReader<MouseMotion>,
    mut camera_query: Query<(&mut CameraSettings, &mut Transform)>,
    time: Res<Time>,
    windows: Res<Windows>,
) {
    let (mut camera_settings, mut camera_transform) = camera_query.get_single_mut().unwrap();

    let cursor_captured = match windows.get_primary() {
        Some(window) => window.cursor_locked(),
        None => false,
    };

    if cursor_captured {
        for mouse_movement in mouse_movement.iter() {
            camera_settings.yaw -=
                mouse_movement.delta.x * time.delta_seconds() * camera_settings.sensitivity;
            camera_settings.pitch = f32::clamp(
                camera_settings.pitch
                    - mouse_movement.delta.y * time.delta_seconds() * camera_settings.sensitivity,
                -0.5 * PI,
                0.5 * PI,
            );
        }
    }

    let rotation_yaw = Quat::from_rotation_y(camera_settings.yaw);
    let rotation_pitch = Quat::from_rotation_x(camera_settings.pitch);

    camera_transform.rotation = rotation_yaw * rotation_pitch;
}
