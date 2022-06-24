use std::f32::EPSILON;

use bevy::{prelude::*, tasks::ComputeTaskPool};
use bevy_inspector_egui::{Inspectable, InspectableRegistry};
use rand::{thread_rng, Rng};

const BATCH_SIZE: usize = 1000;
const MOVEMENT_SPEED: f32 = 10.0;
const NUM_BOIDS: u32 = 4000;
const WORLD_BOUNDS: f32 = 200.0;
const VISION_RADIUS: f32 = 10.0;

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

#[derive(Component, Default, Inspectable)]
struct Boid {
    movement_direction: Vec3,
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
            let other_translations_filter = other_translations.iter().filter(|translation| {
                let other_direction = **translation - transform.translation;
                translation.distance_squared(transform.translation) < VISION_RADIUS.powi(2)
                    && boid
                        .movement_direction
                        .normalize()
                        .dot(other_direction.normalize())
                        > -0.2
            });

            if other_translations_filter.clone().count() < 1 {
                return;
            }

            let new_direction = other_translations_filter
                .fold(Vec3::ZERO, |acc, translation| {
                    // The direction from the other boid to this boid.
                    let direction = transform.translation - *translation;

                    let return_val = match direction.length() > EPSILON {
                        true => acc + direction,
                        false => acc,
                    };

                    return_val
                })
                .normalize()
                * separation_force.magnitude;

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

    boid_query.par_for_each_mut(&pool, BATCH_SIZE, |(transform, mut alignment_force, _)| {
        alignment_force.direction = other_translations_directions
            .iter()
            .filter(|(translation, _)| {
                translation.distance_squared(transform.translation) < VISION_RADIUS * VISION_RADIUS
                    && transform
                        .translation
                        .normalize()
                        .dot(translation.normalize())
                        > -0.2
            })
            .fold(Vec3::ZERO, |acc, (_, direction)| acc + *direction)
            .try_normalize()
            .unwrap_or_else(|| Vec3::splat(1.0));
    });

    /*
    for (transform, mut alignment_force, _) in boid_query.iter_mut() {
        alignment_force.0 = other_translations_directions
            .iter()
            .filter(|(translation, _)| translation.distance_squared(transform.translation) < VISION_RADIUS * VISION_RADIUS)
            .fold(Vec3::ZERO, |acc, (_, direction)| acc + *direction)
            .try_normalize()
            .unwrap_or_else(||Vec3::splat(1.0));
    }*/
}

fn calculate_cohesion_force(
    pool: Res<ComputeTaskPool>,
    mut boid_query: Query<(&Transform, &mut CohesionForce)>,
) {
    let other_translations: Vec<Vec3> = boid_query
        .iter()
        .map(|(transform, _)| transform.translation)
        .collect();

    boid_query.par_for_each_mut(&pool, BATCH_SIZE, |(transform, mut cohesion_force)| {
        cohesion_force.direction = other_translations
            .iter()
            .filter(|translation| {
                transform
                    .translation
                    .normalize()
                    .dot(translation.normalize())
                    > -0.2
            })
            .fold(Vec3::ZERO, |acc, translation| {
                let direction = *translation - transform.translation;

                acc + direction * VISION_RADIUS * direction.length()
            })
            .normalize();
    });

    /*
    for (transform, mut cohesion_force) in boid_query.iter_mut() {
        cohesion_force.0 = other_translations
            .iter()
            .fold(Vec3::ZERO, |acc, translation| {
                let direction = *translation - transform.translation;

                acc + direction * VISION_RADIUS * direction.length()
            })
            .normalize();
    }*/
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
        let force_sum = separation_force.direction; // + alignment_force.direction + cohesion_force.direction;

        let target_direction = force_sum.normalize();
        let target_rotation = Quat::from_rotation_arc(Vec3::Y, target_direction);

        // There are two ways that we can rotate to a new direction,
        // we check if the dot product between the current rotation and the target
        // rotation is positive first to ensure the shortest path.
        // If the dot product isn't positive, then we use the negative of our current rotation.
        let new_rotation = match transform.rotation.dot(target_rotation) >= 0.0 {
            true => transform
                .rotation
                .slerp(target_rotation, time.delta_seconds()),

            false => (-transform.rotation).slerp(target_rotation, time.delta_seconds()),
        };

        boid.movement_direction = new_rotation * Vec3::Y;

        transform.translation += boid.movement_direction * time.delta_seconds() * MOVEMENT_SPEED;
        transform.rotation = new_rotation;
    }
}
