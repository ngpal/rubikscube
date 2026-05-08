use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<OrbitState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera, turn_faces))
        .run();
}

#[derive(Resource, Default)]
struct OrbitState {
    yaw: f32,
    pitch: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Cubes
    for i in -1..=1 {
        for j in -1..=1 {
            for k in -1..=1 {
                let pos = Vec3::new(i as f32, j as f32, k as f32);

                let cube_material = materials.add(Color::BLACK);

                commands
                    .spawn((Transform::from_translation(pos), Visibility::default()))
                    .with_children(|parent| {
                        // Main cube body
                        parent.spawn((
                            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                            MeshMaterial3d(cube_material.clone()),
                            Transform::default(),
                        ));

                        let sticker_size = 0.85;
                        let sticker_depth = 0.05;
                        let offset = 0.5;

                        // +X
                        if i == 1 {
                            parent.spawn((
                                Mesh3d(meshes.add(Cuboid::new(
                                    sticker_depth,
                                    sticker_size,
                                    sticker_size,
                                ))),
                                MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
                                Transform::from_xyz(offset, 0.0, 0.0),
                            ));
                        }

                        // -X
                        if i == -1 {
                            parent.spawn((
                                Mesh3d(meshes.add(Cuboid::new(
                                    sticker_depth,
                                    sticker_size,
                                    sticker_size,
                                ))),
                                MeshMaterial3d(materials.add(Color::srgb(1.0, 0.5, 0.0))),
                                Transform::from_xyz(-offset, 0.0, 0.0),
                            ));
                        }

                        // +Y
                        if j == 1 {
                            parent.spawn((
                                Mesh3d(meshes.add(Cuboid::new(
                                    sticker_size,
                                    sticker_depth,
                                    sticker_size,
                                ))),
                                MeshMaterial3d(materials.add(Color::WHITE)),
                                Transform::from_xyz(0.0, offset, 0.0),
                            ));
                        }

                        // -Y
                        if j == -1 {
                            parent.spawn((
                                Mesh3d(meshes.add(Cuboid::new(
                                    sticker_size,
                                    sticker_depth,
                                    sticker_size,
                                ))),
                                MeshMaterial3d(materials.add(Color::srgb(1., 1., 0.))),
                                Transform::from_xyz(0.0, -offset, 0.0),
                            ));
                        }

                        // +Z
                        if k == 1 {
                            parent.spawn((
                                Mesh3d(meshes.add(Cuboid::new(
                                    sticker_size,
                                    sticker_size,
                                    sticker_depth,
                                ))),
                                MeshMaterial3d(materials.add(Color::srgb(0.0, 1.0, 0.0))),
                                Transform::from_xyz(0.0, 0.0, offset),
                            ));
                        }

                        // -Z
                        if k == -1 {
                            parent.spawn((
                                Mesh3d(meshes.add(Cuboid::new(
                                    sticker_size,
                                    sticker_size,
                                    sticker_depth,
                                ))),
                                MeshMaterial3d(materials.add(Color::srgb(0.0, 0.0, 1.0))),
                                Transform::from_xyz(0.0, 0.0, -offset),
                            ));
                        }
                    });
            }
        }
    }

    // Camera with attached light
    commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 0.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        ))
        .with_children(|parent| {
            parent.spawn((
                PointLight {
                    intensity: 2_000_000.0,
                    shadows_enabled: false,
                    range: 100.0,
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        });
}

fn orbit_camera(
    mut state: ResMut<OrbitState>,
    motion: Res<AccumulatedMouseMotion>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    if !mouse.pressed(MouseButton::Left) {
        return;
    }

    let delta = motion.delta;

    if delta == Vec2::ZERO {
        return;
    }

    let sensitivity = 0.005;

    state.yaw -= delta.x * sensitivity;
    state.pitch += delta.y * sensitivity;

    state.pitch = state.pitch.clamp(-1.5, 1.5);

    let radius = 8.0;

    let x = radius * state.pitch.cos() * state.yaw.sin();
    let y = radius * state.pitch.sin();
    let z = radius * state.pitch.cos() * state.yaw.cos();

    let mut transform = camera.single_mut().unwrap();

    transform.translation = Vec3::new(x, y, z);
    transform.look_at(Vec3::ZERO, Vec3::Y);
}

fn turn_faces(
    mut cubes: Query<&mut Transform, Without<Camera3d>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if !keys.pressed(KeyCode::KeyR) {
        return;
    }

    let angle = 1.0_f32.to_radians() * 60.0 * time.delta_secs();

    for mut transform in &mut cubes {
        // Right face (+X layer)
        if transform.translation.x > 0.9 {
            transform.rotate_around(Vec3::ZERO, Quat::from_rotation_x(-angle));
        }
    }
}
