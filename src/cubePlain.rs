use avian3d::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use smooth_bevy_cameras::{LookTransform, Smoother};

#[derive(Component)]
pub struct CubePlain;

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
        RigidBody::Dynamic,
        Collider::cuboid(0.7, 0.7, 0.7),
        Mesh3d(meshes.add(Cuboid::new(0.7, 0.7, 0.7))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 100.0, 0.0),
        CubePlain,
    ));

    // 平滑摄像机
    let camera_at = CameraLookAt {
        look_at: Vec3::new(0.0, 0.0, 0.0) - Vec3::new(30.0, 18.0, 30.0),
    };
    commands.insert_resource(camera_at);
    commands.spawn((
        LookTransform::new(Vec3::ZERO, Vec3::ZERO, Vec3::Y),
        Smoother::new(0.9),
        Camera3d::default(),
    ));
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
    mut lv_query: Query<&mut LinearVelocity, With<CubePlain>>,
) {
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
    
    let mut velocity: Mut<'_, LinearVelocity> = lv_query.single_mut();
    velocity.x = direction.x * 40.0;
    // velocity.y = look_direction.y;
    velocity.z = direction.z * 40.0;
}

pub fn handle_camera(
    camera_look_at: Res<CameraLookAt>,
    cube_plain_position_query: Query<&Transform, With<CubePlain>>,
    mut look_transform_query: Query<&mut LookTransform>,
) {
    // 更新摄像机位置
    let Ok(mut lt) = look_transform_query.get_single_mut() else {
        return;
    };
    let cube_plain_position: &Transform = cube_plain_position_query.single();
    lt.eye = cube_plain_position.translation - camera_look_at.look_at;
    lt.target = cube_plain_position.translation;
}
