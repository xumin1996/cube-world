use avian3d::prelude::*;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    // 地形
    let perlin = Perlin::new(1);
    for x in 1..10 {
        for z in 1..10 {
            let height = perlin.get([x as f64 / 10.0, z as f64 / 10.0]);
            commands.spawn((
                RigidBody::Static,
                Collider::cuboid(1.0, 1.0, 1.0),
                PbrBundle {
                    mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                    material: materials.add(Color::WHITE),
                    transform: Transform::from_xyz(x as f32, height as f32 * 2.0f32, z as f32),
                    ..default()
                },
            ));
        }
    }

    // 角色
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(0.2, 0.2, 0.2),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.2, 0.2, 0.2)),
            material: materials.add(Color::WHITE),
            transform: Transform::from_xyz(5.0, 10.0, 5.0),
            ..default()
        },
    ));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0)
            .looking_at(Vec3::new(5.0, 0.0, 5.0), Vec3::Y),
        ..default()
    });
}
