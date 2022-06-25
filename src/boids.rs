use std::f32::EPSILON;

use bevy::{prelude::*, tasks::ComputeTaskPool};
use bevy_inspector_egui::{Inspectable, InspectableRegistry};
use rand::{thread_rng, Rng};

const BATCH_SIZE: usize = 1000;
const NUM_BOIDS: u32 = 4000;
const WORLD_BOUNDS: f32 = 200.0;

#[derive(Component, Inspectable)]
struct SeparationForce {
    direction: Vec3,
    magnitude: f32, // A.K.A how much to obey this force.
}

impl Default for SeparationForce {
    fn default() -> Self {
        let mut rng = thread_rng();

        Self {
            direction: Vec3::new(
                rng.gen_range(-1.0..=1.0),
                rng.gen_range(-1.0..=1.0),
                rng.gen_range(-1.0..=1.0),
            )
            .normalize(),

            magnitude: 1.0,
        }
    }
}

#[derive(Component, Inspectable)]
struct AlignmentForce {
    direction: Vec3,
    magnitude: f32, // A.K.A how much to obey this force.
}

impl Default for AlignmentForce {
    fn default() -> Self {
        let mut rng = thread_rng();

        Self {
            direction: Vec3::new(
                rng.gen_range(-1.0..=1.0),
                rng.gen_range(-1.0..=1.0),
                rng.gen_range(-1.0..=1.0),
            )
            .normalize(),

            magnitude: 1.0,
        }
    }
}

#[derive(Component, Inspectable)]
struct CohesionForce {
    direction: Vec3,
    magnitude: f32, // A.K.A how much to obey this force.
}

impl Default for CohesionForce {
    fn default() -> Self {
        let mut rng = thread_rng();

        Self {
            direction: Vec3::new(
                rng.gen_range(-1.0..=1.0),
                rng.gen_range(-1.0..=1.0),
                rng.gen_range(-1.0..=1.0),
            )
            .normalize(),

            magnitude: 1.0,
        }
    }
}

#[derive(Component, Inspectable)]
struct Boid {
    movement_direction: Vec3,
    movement_speed: f32,
    vision_dot: f32,
    vision_distance: f32,
}

impl Default for Boid {
    fn default() -> Self {
        let mut rng = thread_rng();

        Self {
            movement_direction: Vec3::new(
                rng.gen_range(-1.0..=1.0),
                rng.gen_range(-1.0..=1.0),
                rng.gen_range(-1.0..=1.0),
            )
            .normalize(),

            movement_speed: rng.gen_range(10.0..=20.0),
            vision_dot: -0.3 + rng.gen_range(-0.2..=0.2),
            vision_distance: rng.gen_range(10.0..30.0),
        }
    }
}

#[derive(Bundle, Default)]
struct BoidBundle {
    boid: Boid,
    separation_force: SeparationForce,
    alignment_force: AlignmentForce,
    cohesion_force: CohesionForce,
}

pub struct BoidsPlugin;
impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_boids)
            .add_system(calculate_separation_force.before("boid_movement"))
            .add_system(calculate_alignment_force.before("boid_movement"))
            .add_system(calculate_cohesion_force.before("boid_movement"))
            .add_system(move_boids.label("boid_movement"))
            .add_system(wrap_boids.after("boid_movement"));

        if cfg!(debug_assertions) {
            let mut registry = app
                .world
                .get_resource_or_insert_with(InspectableRegistry::default);

            registry.register::<SeparationForce>();
            registry.register::<AlignmentForce>();
            registry.register::<CohesionForce>();
            registry.register::<Boid>();
        }
    }
}

fn spawn_boids(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //let scene = asset_server.load("boid.gltf#Scene0");
    let mesh = asset_server.load("boid.gltf#Mesh0/Primitive0");
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
            .insert_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: materials.add(StandardMaterial {
                    base_color: Color::hsl(
                        rng.gen_range(0.0..=255.0),
                        rng.gen_range(0.6..1.0),
                        0.7,
                    ),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .insert(random_transform)
            .insert(GlobalTransform::default())
            .insert(Name::new("Boid"))
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

fn wrap_boids(mut transform_query: Query<&mut Transform, With<Boid>>) {
    for mut transform in transform_query.iter_mut() {
        let translation = &mut transform.translation;
        if translation.x > WORLD_BOUNDS {
            translation.x = -WORLD_BOUNDS;
        }
        if translation.x < -WORLD_BOUNDS {
            translation.x = WORLD_BOUNDS;
        }
        if translation.y > WORLD_BOUNDS {
            translation.y = -WORLD_BOUNDS;
        }
        if translation.y < -WORLD_BOUNDS {
            translation.y = WORLD_BOUNDS;
        }
        if translation.z > WORLD_BOUNDS {
            translation.z = -WORLD_BOUNDS;
        }
        if translation.z < -WORLD_BOUNDS {
            translation.z = WORLD_BOUNDS;
        }
    }
}

fn calculate_separation_force(
    pool: Res<ComputeTaskPool>,
    mut boid_query: Query<(&Transform, &mut SeparationForce, &Boid)>,
) {
    let other_translations: Vec<Vec3> = boid_query
        .iter()
        .map(|(transform, _, _)| transform.translation)
        .collect();

    boid_query.par_for_each_mut(
        &pool,
        BATCH_SIZE,
        |(transform, mut separation_force, boid)| {
            // Filter out all translations that are too far away from our vision or are behind us.
            let other_translations_filter = other_translations.iter().filter(|translation| {
                let other_direction = **translation - transform.translation;
                translation.distance_squared(transform.translation) < boid.vision_distance.powi(2)
                    && boid
                        .movement_direction
                        .normalize()
                        .dot(other_direction.normalize())
                        > boid.vision_dot
            });

            if other_translations_filter.clone().count() < 1 {
                return;
            }

            let new_direction = other_translations_filter
                .fold(Vec3::ZERO, |acc, translation| {
                    // The direction from the other boid to this boid.
                    let direction = transform.translation - *translation;

                    let return_val = match direction.length() > EPSILON {
                        true => acc + direction * (boid.vision_distance / direction.length()),
                        false => acc,
                    };

                    return_val
                })
                .normalize();

            separation_force.direction = match new_direction.is_nan() {
                true => separation_force.direction,
                false => new_direction,
            }
        },
    );
}

fn calculate_alignment_force(
    pool: Res<ComputeTaskPool>,
    mut boid_query: Query<(&Transform, &mut AlignmentForce, &Boid)>,
) {
    let other_translations_directions: Vec<(Vec3, Vec3)> = boid_query
        .iter()
        .map(|(transform, _, boid)| (transform.translation, boid.movement_direction))
        .collect();

    boid_query.par_for_each_mut(
        &pool,
        BATCH_SIZE,
        |(transform, mut alignment_force, boid)| {
            let other_translations_directions_filter =
                other_translations_directions
                    .iter()
                    .filter(|(translation, _)| {
                        translation.distance_squared(transform.translation)
                            < boid.vision_distance.powi(2)
                            && transform
                                .translation
                                .normalize()
                                .dot(translation.normalize())
                                > boid.vision_dot
                    });

            if other_translations_directions_filter.clone().count() < 1 {
                return;
            }

            alignment_force.direction = other_translations_directions_filter
                .fold(Vec3::ZERO, |acc, (_, direction)| acc + *direction)
                .normalize();
        },
    );
}

fn calculate_cohesion_force(
    pool: Res<ComputeTaskPool>,
    mut boid_query: Query<(&Transform, &mut CohesionForce, &Boid)>,
) {
    let other_translations: Vec<Vec3> = boid_query
        .iter()
        .map(|(transform, _, _)| transform.translation)
        .collect();

    boid_query.par_for_each_mut(
        &pool,
        BATCH_SIZE,
        |(transform, mut cohesion_force, boid)| {
            let other_translations_filter = other_translations.iter().filter(|translation| {
                translation.distance_squared(transform.translation) < boid.vision_distance.powi(2)
                    && transform
                        .translation
                        .normalize()
                        .dot(translation.normalize())
                        > boid.vision_dot
            });

            let other_translations_count = other_translations_filter.clone().count();

            if other_translations_count < 1 {
                return;
            }

            let new_direction =
                other_translations_filter.fold(Vec3::ZERO, |acc, translation| acc + *translation);

            let new_direction =
                new_direction / other_translations_count as f32 - transform.translation;

            let new_direction = new_direction.normalize();

            cohesion_force.direction = match new_direction.is_nan() {
                true => cohesion_force.direction,
                false => new_direction,
            }
        },
    );
}

fn move_boids(
    mut boid_query: Query<(
        &mut Transform,
        &mut Boid,
        &SeparationForce,
        &AlignmentForce,
        &CohesionForce,
    )>,
    time: Res<Time>,
) {
    for (mut transform, mut boid, separation_force, alignment_force, cohesion_force) in
        boid_query.iter_mut()
    {
        let separation_force = separation_force.direction * separation_force.magnitude;
        let alignment_force = alignment_force.direction * alignment_force.magnitude;
        let cohesion_force = cohesion_force.direction * cohesion_force.magnitude;
        let collective_force = separation_force + alignment_force + cohesion_force;

        let target_direction = collective_force.normalize();

        let target_rotation = Quat::from_rotation_arc(Vec3::Y, target_direction);

        // There are two ways that we can rotate to a new direction,
        // we check if the dot product between the current rotation and the target
        // rotation is positive first to ensure the shortest path.
        // If the dot product isn't positive, then we use the negative of our current rotation.
        let new_rotation = match transform.rotation.dot(target_rotation) >= 0.0 {
            true => transform
                .rotation
                .slerp(target_rotation, time.delta_seconds()),

            false => (-transform.rotation).slerp(
                target_rotation,
                time.delta_seconds() * (boid.movement_speed / 10.0),
            ),
        };

        boid.movement_direction = new_rotation * Vec3::Y;

        transform.translation +=
            boid.movement_direction * time.delta_seconds() * boid.movement_speed;
        transform.rotation = new_rotation;
    }
}
