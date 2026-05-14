mod cube;

use bevy::prelude::*;
use bevy::{input::mouse::AccumulatedMouseMotion, window::WindowResolution};
use std::collections::VecDeque;

use crate::cube::{Cube, Move, MoveFace};

#[derive(Resource, Default)]
struct MoveQueue {
    queue: VecDeque<Move>,
    active: Option<ActiveMove>,
}

struct ActiveMove {
    move_type: Move,
    rotated: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rubik's Cube".into(),
                resolution: WindowResolution::new(600, 600),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<OrbitState>()
        .init_resource::<MoveQueue>()
        .init_resource::<Cube>()
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera, handle_input, execute_moves))
        .run();
}

#[derive(Resource, Default)]
struct OrbitState {
    yaw: f32,
    pitch: f32,
}

#[allow(unused)]
fn edges_info(mut cube: ResMut<Cube>) {
    for i in 0..12 {
        let edge = cube.get_edge(i);
        info!("Edge {:X} = {:0>2X}", i, edge)
    }
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

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut moves: ResMut<MoveQueue>,
    mut cube: ResMut<Cube>,
) {
    let prime = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    let face = if keys.just_pressed(KeyCode::KeyR) {
        Some(MoveFace::R)
    } else if keys.just_pressed(KeyCode::KeyL) {
        Some(MoveFace::L)
    } else if keys.just_pressed(KeyCode::KeyU) {
        Some(MoveFace::U)
    } else if keys.just_pressed(KeyCode::KeyD) {
        Some(MoveFace::D)
    } else if keys.just_pressed(KeyCode::KeyF) {
        Some(MoveFace::F)
    } else if keys.just_pressed(KeyCode::KeyB) {
        Some(MoveFace::B)
    } else {
        None
    };

    if let Some(face) = face {
        let m = Move::new(face, prime, false);
        moves.queue.push_back(m);
        cube.make_move(m);
        edges_info(cube);
    }
}

fn execute_moves(
    mut cubes: Query<&mut Transform, Without<Camera3d>>,
    time: Res<Time>,
    mut moves: ResMut<MoveQueue>,
) {
    if moves.active.is_none() {
        if let Some(next_move) = moves.queue.pop_front() {
            moves.active = Some(ActiveMove {
                move_type: next_move,
                rotated: 0.0,
            });
        }
    }

    let Some(active) = &mut moves.active else {
        return;
    };

    let speed = 180.0_f32.to_radians();
    let mut step = speed * time.delta_secs();
    let remaining = 90.0_f32.to_radians() - active.rotated;

    if step > remaining {
        step = remaining;
    }

    match active.move_type {
        Move {
            face: MoveFace::R,
            is_prime,
            is_double,
        } => {
            let dir = if is_prime {
                1.0
            } else if is_double {
                -2.0
            } else {
                -1.0
            };
            for mut transform in &mut cubes {
                if transform.translation.x > 0.9 {
                    transform.rotate_around(Vec3::ZERO, Quat::from_rotation_x(dir * step));
                }
            }
        }
        Move {
            face: MoveFace::L,
            is_prime,
            is_double,
        } => {
            let dir = if is_prime {
                -1.0
            } else if is_double {
                2.0
            } else {
                1.0
            };
            for mut transform in &mut cubes {
                if transform.translation.x < -0.9 {
                    transform.rotate_around(Vec3::ZERO, Quat::from_rotation_x(dir * step));
                }
            }
        }
        Move {
            face: MoveFace::U,
            is_prime,
            is_double,
        } => {
            let dir = if is_prime {
                1.0
            } else if is_double {
                -2.0
            } else {
                -1.0
            };
            for mut transform in &mut cubes {
                if transform.translation.y > 0.9 {
                    transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(dir * step));
                }
            }
        }
        Move {
            face: MoveFace::D,
            is_prime,
            is_double,
        } => {
            let dir = if is_prime {
                -1.0
            } else if is_double {
                2.0
            } else {
                1.0
            };
            for mut transform in &mut cubes {
                if transform.translation.y < -0.9 {
                    transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(dir * step));
                }
            }
        }
        Move {
            face: MoveFace::F,
            is_prime,
            is_double,
        } => {
            let dir = if is_prime {
                1.0
            } else if is_double {
                -2.0
            } else {
                -1.0
            };
            for mut transform in &mut cubes {
                if transform.translation.z > 0.9 {
                    transform.rotate_around(Vec3::ZERO, Quat::from_rotation_z(dir * step));
                }
            }
        }
        Move {
            face: MoveFace::B,
            is_prime,
            is_double,
        } => {
            let dir = if is_prime {
                -1.0
            } else if is_double {
                2.0
            } else {
                1.0
            };
            for mut transform in &mut cubes {
                if transform.translation.z < -0.9 {
                    transform.rotate_around(Vec3::ZERO, Quat::from_rotation_z(dir * step));
                }
            }
        }
    }

    active.rotated += step;

    if active.rotated >= 90.0_f32.to_radians() - 0.0001 {
        moves.active = None;
    }
}
