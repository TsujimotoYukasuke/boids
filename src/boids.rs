use bevy::prelude::*;
use rand::{thread_rng, Rng};

const NUM_BOIDS: u32 = 1000;
const WORLD_BOUNDS: f32 = 100.0;

pub struct BoidsPlugin;
impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_boids);
    }
}

fn spawn_boids(mut commands: Commands, asset_server: Res<AssetServer>) {
    let scene = asset_server.load("boid.gltf#Scene0");
    let mut boids = Vec::new();

    for _ in 0..NUM_BOIDS {
        let mut rng = thread_rng();
        let random_transform = Transform::from_xyz(
            rng.gen_range(-WORLD_BOUNDS..=WORLD_BOUNDS),
            rng.gen_range(-WORLD_BOUNDS..=WORLD_BOUNDS),
            rng.gen_range(-WORLD_BOUNDS..=WORLD_BOUNDS),
        );

        let boid = commands
            .spawn()
            .insert(random_transform)
            .insert(GlobalTransform::default())
            .insert(Name::new("Boid"))
            .with_children(|parent| {
                parent.spawn_scene(scene.clone());
            })
            .id();

        boids.push(boid);
    }

    commands
        .spawn()
        .insert(Name::new("Boids"))
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .push_children(&boids);
}
