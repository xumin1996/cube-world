use avian3d::{parry::shape, prelude::*};
use bevy::input::mouse::MouseMotion;
use bevy::math::VectorSpace;
use bevy::prelude::*;
use bevy_tnua::prelude::*;
use smooth_bevy_cameras::{LookTransform, LookTransformBundle, LookTransformPlugin, Smoother};

#[derive(Component)]
pub struct Player;

#[derive(Resource)]
pub struct CameraLookAt {
    pub look_at: Vec3,
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 角色
    commands.spawn((
        TnuaControllerBundle::default(),
        RigidBody::Dynamic,
        Collider::cuboid(0.2, 0.2, 0.2),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.8, 0.8, 0.8)),
            material: materials.add(Color::WHITE),
            transform: Transform::from_xyz(5.0, 15.0, 5.0),
            ..default()
        },
        Player,
    ));

    // 点光源
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            range: 100.0,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 10.0, 4.0),
        ..default()
    });

    // 平滑摄像机
    let camera_at = CameraLookAt {
        look_at: Vec3::new(0.0, 0.0, 0.0) - Vec3::new(10.0, 6.0, 10.0),
    };
    commands.insert_resource(camera_at);
    commands
        .spawn(LookTransformBundle {
            transform: LookTransform::new(Vec3::ZERO, Vec3::ZERO, Vec3::Y),
            smoother: Smoother::new(0.9),
        })
        .insert(Camera3dBundle::default());
}

pub fn handle_mouse_motion(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera_look_at: ResMut<CameraLookAt>,
) {
    let displacement = mouse_motion_events
        .read()
        .fold(0., |acc, mouse_motion| acc + mouse_motion.delta.x);

    // 旋转
    let mut camera_transform = Transform::from_translation(camera_look_at.look_at);
    camera_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(-displacement / 500.));
    camera_look_at.look_at = Vec3::new(
        camera_transform.translation.x,
        camera_transform.translation.y,
        camera_transform.translation.z,
    );
}

pub fn handle_keyboard_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_look_at: Res<CameraLookAt>,
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
        desired_velocity: direction.normalize_or_zero() * 15.0,
        float_height: 1.5,
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
    lt.target = player_position.translation;
}

pub fn handle_camera(
    camera_look_at: Res<CameraLookAt>,
    mut player_position_query: Query<&mut Transform, With<Player>>,
    mut lookTransformQuery: Query<&mut LookTransform>,
) {
    // 更新摄像机位置
    let Ok(mut lt) = lookTransformQuery.get_single_mut() else {
        return;
    };
    let player_position: &Transform = player_position_query.get_single().unwrap();
    lt.eye = player_position.translation - camera_look_at.look_at;
    lt.target = player_position.translation;
}
