use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<OrbitState>()
        .add_systems(Startup, setup)
        .add_systems(Update, orbit_camera)
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
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                    MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.2))),
                    Transform::from_xyz(i as f32 * 1.1, j as f32 * 1.1, k as f32 * 1.1),
                ));
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
