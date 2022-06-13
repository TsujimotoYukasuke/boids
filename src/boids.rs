use bevy::prelude::*;
use rand::{thread_rng, Rng};

const MOVEMENT_SPEED: f32 = 5.0;
const NUM_BOIDS: u32 = 1000;
const WORLD_BOUNDS: f32 = 100.0;
const VISION_RADIUS: f32 = 20.0;

#[derive(Component, Default)]
struct SeparationForce(Vec3);

#[derive(Component, Default)]
struct AlignmentForce(Vec3);

#[derive(Component, Default)]
struct CohesionForce(Vec3);

#[derive(Component, Default)]
struct WorldForce(Vec3);

#[derive(Component, Default)]
struct Boid {
    movement_direction: Vec3,
}

#[derive(Bundle, Default)]
struct BoidBundle {
    boid: Boid,
    separation_force: SeparationForce,
    alignment_force: AlignmentForce,
    cohesion_force: CohesionForce,
    world_force: WorldForce,
}

pub struct BoidsPlugin;
impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_boids)
            .add_system(calculate_separation_force.before("boid_movement"))
            .add_system(calculate_alignment_force.before("boid_movement"))
            .add_system(calculate_cohesion_force.before("boid_movement"))
            .add_system(calculate_world_force.before("boid_movement"))
            .add_system(move_boid.label("boid_movement"));
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
            .spawn_bundle(BoidBundle::default())
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

fn calculate_separation_force(
    boid_query: Query<(&Transform, &Boid)>,
    mut separation_force_query: Query<(Entity, &mut SeparationForce)>,
) {
    for (entity, mut separation_force) in separation_force_query.iter_mut() {
        let (this_transform, this_boid) = boid_query.get(entity).unwrap();
        let this_translation = this_transform.translation;

        let relative_locations: Vec<Vec3> = boid_query
            .iter()
            .map(|(transform, _)| transform.translation - this_translation)
            .filter(|translation| {
                let distance = translation.distance_squared(this_translation);
                let dot = Vec3::dot(this_boid.movement_direction, translation.normalize());

                distance > f32::EPSILON.powi(2) && distance <= VISION_RADIUS.powi(2) && dot > 0.0
            })
            .collect();

        separation_force.0 = Vec3::ZERO;
        for location in relative_locations {
            let force_magnitude = VISION_RADIUS / 10.0 / location.distance(this_translation);
            separation_force.0 += (this_translation - location).normalize() * force_magnitude;
        }
    }
}

fn calculate_alignment_force(
    boid_query: Query<(&Transform, &Boid)>,
    mut alignment_force_query: Query<(Entity, &mut AlignmentForce)>,
) {
    for (entity, mut alignment_force) in alignment_force_query.iter_mut() {
        let (this_transform, this_boid) = boid_query.get(entity).unwrap();
        let this_translation = this_transform.translation;

        let relative_directions: Vec<Vec3> = boid_query
            .iter()
            .map(|(transform, boid)| {
                (
                    transform.translation - this_translation,
                    boid.movement_direction,
                )
            })
            .filter(|(translation, _)| {
                let distance = translation.distance_squared(this_translation);
                let dot = Vec3::dot(this_boid.movement_direction, translation.normalize());

                distance > f32::EPSILON.powi(2) && distance <= VISION_RADIUS.powi(2) && dot > 0.0
            })
            .map(|(_, movement_direction)| movement_direction)
            .collect();

        alignment_force.0 = Vec3::ZERO;
        for direction in relative_directions {
            alignment_force.0 += direction;
        }
    }
}

fn calculate_cohesion_force(
    boid_query: Query<(&Transform, &Boid)>,
    mut cohesion_force_query: Query<(Entity, &mut CohesionForce)>,
) {
    for (entity, mut cohesion_force) in cohesion_force_query.iter_mut() {
        let (this_transform, this_boid) = boid_query.get(entity).unwrap();
        let this_translation = this_transform.translation;

        cohesion_force.0 = boid_query
            .iter()
            .map(|(transform, _)| transform.translation - this_translation)
            .filter(|translation| this_boid.movement_direction.dot(translation.normalize()) > 0.0)
            .reduce(|acc, value| acc + value)
            .unwrap_or(Vec3::ZERO)
            / (NUM_BOIDS - 1) as f32;
    }
}

fn calculate_world_force(mut boid_query: Query<(&Transform, &mut WorldForce), With<Boid>>) {
    for (transform, mut world_force) in boid_query.iter_mut() {
        if transform.translation.length() >= WORLD_BOUNDS / 2.0 {
            world_force.0 = Vec3::ZERO - transform.translation * transform.translation.length() / 2.0;
        }
    }
}

fn move_boid(
    mut boid_query: Query<(
        &mut Transform,
        &mut Boid,
        &SeparationForce,
        &AlignmentForce,
        &CohesionForce,
        &WorldForce,
    )>,
    time: Res<Time>,
) {
    for (mut transform, mut boid, separation_force, alignment_force, cohesion_force, world_force) in
        boid_query.iter_mut()
    {
        boid.movement_direction =
            (separation_force.0 + alignment_force.0 + cohesion_force.0 + world_force.0).normalize();

        let cross = Vec3::cross(boid.movement_direction, Vec3::Y);
        let w = f32::sqrt(boid.movement_direction.length().powi(2) * Vec3::Y.length().powi(2) + Vec3::dot(boid.movement_direction, Vec3::Y));

        transform.translation += boid.movement_direction * time.delta_seconds() * MOVEMENT_SPEED;
        transform.rotation = Quat::from_xyzw(cross.x, cross.y, cross.z, w).normalize();
    }
}
