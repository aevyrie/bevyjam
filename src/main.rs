use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_atmosphere::*;
use heron::prelude::*;
use ringbuffer::{ConstGenericRingBuffer, RingBufferExt, RingBufferWrite};

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(AtmospherePlugin { dynamic: true })
        .insert_resource(AtmosphereMat::default())
        .insert_resource(WindowDescriptor {
            vsync: true,
            ..Default::default()
        })
        .insert_resource(Gravity::from(Vec3::new(0.0, -9.81, 0.0)))
        .insert_resource(ParticleParams::default())
        .add_startup_system(setup)
        .add_system(particles)
        .add_system(daylight_cycle)
        .run();
}

#[derive(Debug, Default)]
struct ParticleParams {
    radius: f32,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    ringbuffer: ConstGenericRingBuffer<Entity, MAX_PARTICLES>,
}

const MAX_PARTICLES: usize = 512;

#[derive(Component)]
struct Particle;

fn particles(mut commands: Commands, mut params: ResMut<ParticleParams>) {
    for _ in 0..1 {
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
    particle.radius = 0.05;
    particle.mesh = meshes.add(Mesh::from(shape::Icosphere {
        radius: particle.radius,
        subdivisions: 1,
    }));
    particle.material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.1, 0.1, 0.1),
        emissive: Color::rgb(0.9, 0.6, 0.0),
        perceptual_roughness: 0.5,
        metallic: 0.1,
        reflectance: 0.9,
        ..Default::default()
    });

    spawn_ground(&mut commands, &mut meshes, &mut materials);

    let size = 500.0;

    commands
        .spawn_bundle(DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                illuminance: 20000.0,
                color: Color::rgb_u8(201, 226, 255),
                shadow_projection: OrthographicProjection {
                    left: -size,
                    right: size,
                    bottom: -size,
                    top: size,
                    near: -size,
                    far: size,
                    ..Default::default()
                },
                ..Default::default()
            },
            transform: Transform::from_xyz(1.0, 2.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(Sun);
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-100.0, 50.0, -100.0)
            .looking_at(Vec3::new(0.0, 30.0, 0.0), Vec3::Y),
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
            restitution: 0.3,
            density: 0.01,
            friction: 0.1,
        })
        .insert_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 10000.0 * scale * particle.radius,
                range: 100.0 * particle.radius * scale,
                shadows_enabled: false,
                color: Color::rgb(1.0, 0.7, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Transform {
            translation: Vec3::new(
                fastrand::f32() * spread - spread / 2.0,
                fastrand::f32() * spread - spread / 2.0 + 40.0,
                fastrand::f32() * spread - spread / 2.0,
            ),
            scale: Vec3::splat(scale),
            ..Default::default()
        })
        .insert(Velocity::from_linear(Vec3::new(
            (fastrand::f32() - 0.5) * 50.0,
            -5.0,
            (fastrand::f32() - 0.5) * 50.0,
        )))
        .insert(Particle);
}

fn spawn_ground(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.5, 0.5, 0.5),
        perceptual_roughness: 0.7,
        metallic: 0.1,
        reflectance: 0.5,
        ..Default::default()
    });
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
                mesh: meshes.add(Mesh::from(shape::Plane { size: 10000.0 })),
                material: material.clone(),
                transform: Transform::from_xyz(0.0, 100.0, 0.0),
                ..Default::default()
            });
        });

    let obstacle_mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

    for _ in 0..1000 {
        let height = fastrand::f32() * 20.0;
        commands
            .spawn_bundle(PbrBundle {
                mesh: obstacle_mesh.clone(),
                material: material.clone(),
                transform: Transform {
                    translation: Vec3::new(
                        fastrand::f32() * 500.0 - 250.0,
                        height / 2.0,
                        fastrand::f32() * 500.0 - 250.0,
                    ),
                    scale: Vec3::new(5.0, height, 5.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(RigidBody::Static)
            .insert(CollisionShape::Cuboid {
                // let the size be consistent with our sprite
                half_extends: Vec3::new(5.0 / 2.0, height / 2.0, 5.0 / 2.0),
                border_radius: None,
            })
            .insert(PhysicMaterial {
                restitution: 0.9,
                density: 1.0,
                friction: 0.1,
            });
    }
}

// Marker for updating the position of the light, not needed unless we have multiple lights
#[derive(Component)]
struct Sun;

// We can edit the SkyMaterial resource and it will be updated automatically, as long as ZephyrPlugin.dynamic is true
fn daylight_cycle(
    mut sky_mat: ResMut<AtmosphereMat>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    time: Res<Time>,
) {
    let mut pos = sky_mat.sun_position;
    let t = time.time_since_startup().as_millis() as f32 / 5000.0;
    pos.x = t.sin();
    pos.z = t.cos();
    pos.y = t.sin() * 0.5;
    sky_mat.sun_position = pos;

    if let Some((mut light_trans, mut directional)) = query.single_mut().into() {
        light_trans.rotation = Transform::from_translation(pos)
            .looking_at(Vec3::ZERO, Vec3::Y)
            .rotation;
        directional.illuminance = pos.y.max(0.0).powf(2.0) * 100000.0;
    }
}
