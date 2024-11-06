use avian3d::prelude::*;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use smooth_bevy_cameras::{LookTransform, LookTransformBundle, LookTransformPlugin, Smoother};
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use bevy_tnua::prelude::*;


fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            LookTransformPlugin,
            TnuaControllerPlugin::default(),
            TnuaAvian3dPlugin::default()
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, (move_camera_system, player_controller))
        .run();
}

#[derive(Component)]
struct Player;

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
        TnuaControllerBundle::default(),
        RigidBody::Dynamic,
        Collider::cuboid(0.2, 0.2, 0.2),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.2, 0.2, 0.2)),
            material: materials.add(Color::WHITE),
            transform: Transform::from_xyz(5.0, 100.0, 5.0),
            ..default()
        },
    ))
    .insert(Player);

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 10.0, 4.0),
        ..default()
    });

    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_xyz(-2.5, 4.5, 9.0)
    //         .looking_at(Vec3::new(5.0, 0.0, 5.0), Vec3::Y),
    //     ..default()
    // });

    let eye = Vec3::new(-2.5, 4.5, 9.0);
    let look_at = Vec3::new(5.0, 0.0, 5.0);
    commands
        .spawn(LookTransformBundle {
            transform: LookTransform::new(eye, look_at, Vec3::Y),
            smoother: Smoother::new(0.9),
        })
        .insert(Camera3dBundle::default());
}

fn move_camera_system(mut cameras: Query<&mut LookTransform>, player_transform: Query<&mut Transform, With<Player>>) {
    for mut camera in cameras.iter_mut() {
        camera.target = player_transform.single().translation;
    }
}

fn player_controller(mut tnue_controller: Query<&mut TnuaController>) {
 for mut controller in tnue_controller.iter_mut() {
    controller.basis(
        TnuaBuiltinWalk{
            float_height: 5.0,
            ..default()
        }
    );
    controller.action(
        TnuaBuiltinJump{
            height: 10.0,
            ..default()
        }
    );
 }
}
