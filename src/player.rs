use avian3d::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy_tnua::prelude::*;
use smooth_bevy_cameras::{LookTransform, LookTransformBundle, Smoother};

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
            mesh: meshes.add(Cuboid::new(0.7, 0.7, 0.7)),
            material: materials.add(Color::WHITE),
            transform: Transform::from_xyz(5.0, 40.0, 5.0),
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
        float_height: 2.0,
        ..Default::default()
    });

    if keyboard.pressed(KeyCode::Space) {
        controller.action(TnuaBuiltinJump {
            height: 2.0,
            ..Default::default()
        });
    }
}

pub fn handle_camera(
    camera_look_at: Res<CameraLookAt>,
    player_position_query: Query<&Transform, With<Player>>,
    mut look_transform_query: Query<&mut LookTransform>,
) {
    // 更新摄像机位置
    let Ok(mut lt) = look_transform_query.get_single_mut() else {
        return;
    };
    let player_position: &Transform = player_position_query.single();
    lt.eye = player_position.translation - camera_look_at.look_at;
    lt.target = player_position.translation;
}

pub fn handle_light(
    player_position_query: Query<&Transform, With<Player>>,
    mut point_light_transform: Query<&mut Transform, (With<PointLight>, Without<Player>)>,
) {
    let player_position: &Transform = player_position_query.single();
    // 更新光源
    point_light_transform.single_mut().translation = Vec3::new(
        player_position.translation.x,
        player_position.translation.y + 5f32,
        player_position.translation.z,
    );
}
