use bevy::prelude::*;
use noise::{Perlin, NoiseFn};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, camera_controller)
        .run();
}

fn camera_controller(
    mut camera: Query<&mut Transform, With<Camera>>, 
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>
) {
    println!("{:?}", time.delta());

    if keyboard.pressed(KeyCode::KeyW) {
        println!("W")
    }
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(0.5)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });

    for x in 0..100 {
        for y in 0..100 {
            let perlin = Perlin::new(123);
            let height: f64 = perlin.get([x as f64 / 10.0, y as f64 / 10.0]);
            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                material: materials.add(Color::WHITE),
                transform: Transform::from_xyz(x as f32, (height * 10.0 )as f32, y as f32),
                ..default()
            });
        }
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity:1000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(50.0, 50.0,  50.0),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 20.0, 9.0).looking_at(Vec3::new(50.0, 0.0, 50.0), Vec3::Y),
        ..default()
    });
}
