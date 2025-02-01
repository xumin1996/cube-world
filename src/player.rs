use bevy_rapier3d::prelude::*;
use bevy::gltf::{Gltf, GltfMesh, GltfNode};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::state::commands;
use bevy_tnua::prelude::*;
use smooth_bevy_cameras::{LookTransform, Smoother};

#[derive(Component)]
pub struct Player;

#[derive(Resource)]
pub struct CameraLookAt {
    pub look_at: Vec3,
}

#[derive(Resource)]
pub struct MyAssetPacket(Handle<Gltf>);

// 定义按键冷却时间的组件
#[derive(Component)]
pub struct KeyCooldownTimer(Timer);

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // 角色
    commands.spawn((
        TnuaController::default(),
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(5.0, 10.0, 5.0),
        Player,
    ));

    // 点光源
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            range: 100.0,
            ..default()
        },
        Transform::from_xyz(4.0, 10.0, 4.0),
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

    // todo 投掷物品
    let pokeball_handle = asset_server.load("models/pokeball.glb");
    commands.insert_resource(MyAssetPacket(pokeball_handle));

    // 按键冷却时间
    commands.spawn(KeyCooldownTimer(Timer::from_seconds(0.3, TimerMode::Once)));
}

pub fn handle_mouse_motion(
    mut commands: Commands,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera_look_at: ResMut<CameraLookAt>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mouse: Res<ButtonInput<MouseButton>>,
    my_asset_packet: Res<MyAssetPacket>,
    gltf_asset: Res<Assets<Gltf>>,
    gltf_node_asset: Res<Assets<GltfNode>>,
    gltf_mesh_asset: Res<Assets<GltfMesh>>,
    player_position_query: Query<&Transform, With<Player>>,
    mut key_cool_timer_query: Query<&mut KeyCooldownTimer>,
    time: Res<Time>,
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

    // 鼠标
    let key_cool_timer = &mut key_cool_timer_query.single_mut().0;
    key_cool_timer.tick(time.delta());
    if mouse.pressed(MouseButton::Left) && key_cool_timer.finished() {
        if let Some(obj_mesh) = gltf_asset
            .get(&my_asset_packet.0)
            .and_then(|gltf| gltf_node_asset.get(&gltf.named_nodes["SM_Pokeball_M_Pokeball_0"]))
            .and_then(|floor_dirt| floor_dirt.mesh.as_ref())
            .and_then(|floor_mesh_handle| gltf_mesh_asset.get(floor_mesh_handle))
        {
            // 重置冷却时间
            key_cool_timer.reset();

            commands.spawn((
                // Mesh3d(obj_mesh.primitives[0].mesh.clone()),
                // MeshMaterial3d(obj_mesh.primitives[0].material.clone().unwrap()),
                Mesh3d(meshes.add(Sphere::new(1.0))),
                MeshMaterial3d(materials.add(Color::WHITE)),
                RigidBody::Dynamic,
                Collider::ball(1.0),
                // ColliderConstructor::ConvexHullFromMesh,
                // GravityScale(1.0),
                Transform::from_translation(player_position_query.single().translation.clone()),
            ));
        }
    }
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
        float_height: 3.0,
        ..Default::default()
    });

    if keyboard.pressed(KeyCode::Space) {
        controller.action(TnuaBuiltinJump {
            height: 3.0,
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
    mut gizmos: Gizmos,
    player_position_query: Query<&Transform, With<Player>>,
    mut point_light_transform: Query<&mut Transform, (With<PointLight>, Without<Player>)>,
    point_light: Query<&PointLight>,
) {
    let player_position: &Transform = player_position_query.single();
    // 更新光源
    point_light_transform.single_mut().translation = Vec3::new(
        player_position.translation.x,
        player_position.translation.y + 5f32,
        player_position.translation.z,
    );

    // gizmos.sphere(
    //     point_light_transform.single_mut().translation,
    //     Quat::from_rotation_x(0f32),
    //     point_light.single().range,
    //     Color::srgb(1.0, 0f32, 0f32),
    // );
}
