mod boids;
mod camera;
mod debug;

use bevy::prelude::*;
use boids::BoidsPlugin;
use camera::PlayerCameraPlugin;
use debug::DebugPlugin;

const CLEAR_COLOR: Color = Color::rgb(0.0, 0.05, 0.1);
const WINDOW_WIDTH: f32 = 1600.0;
const WINDOW_HEIGHT: f32 = 900.0;

// NOTE: Interestingly, when we decide to just allow all the different forces to have equal weight, our boids try incredibly hard to merge into one another.
fn main() {
    App::new()
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
        .add_plugin(PlayerCameraPlugin)
        .add_startup_system(spawn_light)
        .run();
}

fn spawn_light(mut commands: Commands) {
    commands
        .spawn_bundle(DirectionalLightBundle::default())
        .insert(Name::new("Sun Light"));
}
