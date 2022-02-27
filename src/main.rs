use bevy::{ecs::system::EntityCommands, prelude::*};
use heron::prelude::*;
use ringbuffer::{ConstGenericRingBuffer, RingBufferExt, RingBufferWrite};

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(WindowDescriptor {
            vsync: true,
            ..Default::default()
        })
        .insert_resource(Gravity::from(Vec3::new(0.0, -9.81, 0.0)))
        .insert_resource(ParticleParams::default())
        .add_startup_system(setup)
        .add_system(particles)
        .run();
}

#[derive(Debug, Default)]
struct ParticleParams {
    radius: f32,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    ringbuffer: ConstGenericRingBuffer<Entity, MAX_PARTICLES>,
}

const MAX_PARTICLES: usize = 256;

#[derive(Component)]
struct Particle;

fn particles(mut commands: Commands, mut params: ResMut<ParticleParams>) {
    for _ in 0..2 {
        if let Some(&entity) = params.ringbuffer.get(0) {
            commands.get_or_spawn(entity).despawn_recursive();
        }
        let mut e = commands.spawn();
        spawn_particles(&mut e, &params);
        params.ringbuffer.push(e.id());
    }
}
/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut particle: ResMut<ParticleParams>,
) {
    particle.radius = 0.01;
    particle.mesh = meshes.add(Mesh::from(shape::Icosphere {
        radius: particle.radius,
        subdivisions: 1,
    }));
    particle.material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.1, 0.1, 0.1),
        emissive: Color::rgb(0.8, 0.0, 0.0),
        perceptual_roughness: 0.5,
        metallic: 0.1,
        reflectance: 0.9,
        ..Default::default()
    });

    spawn_ground(&mut commands, &mut meshes, &mut materials);

    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            color: Color::rgb(1.0, 1.0, 1.0),
            ..Default::default()
        },
        transform: Transform::from_xyz(1.0, 2.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-3.0, 2.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn spawn_particles(commands: &mut EntityCommands, particle: &ResMut<ParticleParams>) {
    let scale = 1.0 + fastrand::f32();
    let spread = 0.1;
    commands
        .insert_bundle(PbrBundle {
            mesh: particle.mesh.clone(),
            material: particle.material.clone(),
            ..Default::default()
        })
        .insert(RigidBody::Dynamic)
        // Attach a collision shape
        .insert(CollisionShape::Sphere {
            // let the size be consistent with our sprite
            radius: particle.radius * scale,
        })
        .insert(PhysicMaterial {
            restitution: 0.9,
            density: 1.0,
            friction: 0.1,
        })
        .insert_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 0.2 * scale,
                range: 0.2,
                shadows_enabled: false,
                color: Color::rgb(1.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Transform {
            translation: Vec3::new(
                fastrand::f32() * spread - spread / 2.0,
                fastrand::f32() * spread - spread / 2.0 + 0.1,
                fastrand::f32() * spread - spread / 2.0,
            ),
            scale: Vec3::splat(scale),
            ..Default::default()
        })
        .insert(Velocity::from_linear(Vec3::new(
            (fastrand::f32() - 0.5) * 3.0,
            3.0,
            (fastrand::f32() - 0.5) * 3.0,
        )))
        .insert(Particle);
}

fn spawn_ground(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, -100.0, 0.0))
        .insert(GlobalTransform::default())
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(50.0, 100.0, 50.0),
            border_radius: None,
        })
        .insert(RigidBody::Static) // Attach a collision shape
        .insert(PhysicMaterial {
            restitution: 0.5,
            ..Default::default()
        })
        .with_children(|child| {
            child.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 100.0 })),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.2, 0.2, 0.2),
                    perceptual_roughness: 0.9,
                    metallic: 0.9,
                    reflectance: 0.8,
                    ..Default::default()
                }),
                transform: Transform::from_xyz(0.0, 100.0, 0.0),
                ..Default::default()
            });
        });
}
