use avian3d::{parry::shape, prelude::*};
use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use noise::{NoiseFn, Perlin};
use smooth_bevy_cameras::{LookTransform, LookTransformBundle, LookTransformPlugin, Smoother};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            LookTransformPlugin,
            TnuaControllerPlugin::default(),
            TnuaAvian3dPlugin::default(),
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, (move_camera_system, apply_controls))
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Resource)]
struct CamereLookAt {
    look_at: Vec3
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 地形
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let cube_material = materials.add(Color::WHITE);
    
    let perlin = Perlin::new(1);
    for x in 1..15 {
        for z in 1..15 {
            let height = perlin.get([x as f64 / 10.0, z as f64 / 10.0]);
            commands.spawn((
                RigidBody::Static,
                Collider::cuboid(1.0, 1.0, 1.0),
                PbrBundle {
                    mesh: cube_mesh.clone(),
                    material: cube_material.clone(),
                    transform: Transform::from_xyz(x as f32, height as f32 * 2.0f32, z as f32),
                    ..default()
                },
            ));
        }
    }

    // 角色
    commands
        .spawn((
            TnuaControllerBundle::default(),
            RigidBody::Dynamic,
            Collider::cuboid(0.2, 0.2, 0.2),
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.2, 0.2, 0.2)),
                material: materials.add(Color::WHITE),
                transform: Transform::from_xyz(5.0, 5.0, 5.0),
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

    commands.insert_resource(CamereLookAt{
        look_at: Vec3::Z * 5.0
    });
}

fn move_camera_system(
    mut cameras: Query<&mut LookTransform>,
    player_transform: Query<&mut Transform, With<Player>>,
) {
    for mut camera in cameras.iter_mut() {
        camera.target = player_transform.single().translation;
    }
}

fn apply_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_look_at: Res<CamereLookAt>,
    mut query: Query<&mut TnuaController>,
    mut player_position_query: Query<&mut Transform, With<Player>>,
    mut lookTransformQuery: Query<&mut LookTransform>,
) {
    let Ok(mut controller) = query.get_single_mut() else {
        return;
    };

    let look_direction = camera_look_at.look_at.normalize();
    let rotation_quaternion = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2);
    let look_direction_rotation = rotation_quaternion.mul_vec3(look_direction);

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction += look_direction;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction -= look_direction;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction -= look_direction_rotation;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += look_direction_rotation;
    }
    controller.basis(TnuaBuiltinWalk {
        desired_velocity: direction.normalize_or_zero() * 7.0,
        float_height: 1.0,
        ..Default::default()
    });

    if keyboard.pressed(KeyCode::Space) {
        controller.action(TnuaBuiltinJump {
            height: 1.5,
            ..Default::default()
        });
    }

    // 更新摄像机位置
    let Ok(mut lt) = lookTransformQuery.get_single_mut() else {
        return;
    };
    let player_position: &Transform = player_position_query.get_single().unwrap();
    lt.eye = player_position.translation - camera_look_at.look_at;
}
