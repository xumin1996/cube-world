use std::f32::consts::PI;

use bevy::gltf::{Gltf, GltfMesh, GltfNode};
use bevy::input::mouse::MouseMotion;
use bevy::math::VectorSpace;
use bevy::prelude::*;
use bevy::render::render_resource::Texture;
use bevy::state::commands;
use bevy_rapier3d::prelude::*;
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

#[derive(Component)]
pub struct PrintTimer(Timer);

#[derive(Component, Debug)]
pub struct Bullet {
    live_time: Timer,
}

const collider_player: Group = Group::GROUP_1;
const collider_ground: Group = Group::GROUP_2;
const collider_ball: Group = Group::GROUP_3;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // 角色
    commands.spawn((
        KinematicCharacterController {
            // up: Vec3::Y,
            apply_impulse_to_dynamic_bodies: true,
            offset: CharacterLength::Absolute(0.01),
            snap_to_ground: Some(CharacterLength::Absolute(1.0)),
            autostep: Some(CharacterAutostep {
                max_height: CharacterLength::Absolute(1.5),
                min_width: CharacterLength::Absolute(0.6),
                include_dynamic_bodies: true,
            }),
            ..default()
        },
        LockedAxes::ROTATION_LOCKED,
        RigidBody::Dynamic,
        Collider::ball(0.5),
        GravityScale(4.0),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        // SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/Fox.glb"))),
        Transform::from_xyz(0.0, 10.0, 0.0),
        Player,
        Velocity::zero(),
        // CollisionGroups::new(collider_player, collider_ground),
    ));

    // 平滑摄像机
    let camera_at = CameraLookAt {
        look_at: Vec3::new(0.0, 0.0, 0.0) - Vec3::new(20.0, 12.0, 0.0),
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
    commands.spawn(KeyCooldownTimer(Timer::from_seconds(0.1, TimerMode::Once)));
    commands.spawn(PrintTimer(Timer::from_seconds(1.0, TimerMode::Once)));
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
    mut player_position_query: Query<&mut Transform, With<Player>>,
    // bullet_query: Query<&Transform, With<Bullet>>,
    mut key_cool_timer_query: Query<&mut KeyCooldownTimer>,
    mut print_timer_query: Query<&mut PrintTimer>,
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

    player_position_query
        .single_mut()
        .rotate_local_y(-displacement / 500.);

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
                Bullet {
                    live_time: Timer::from_seconds(10.0, TimerMode::Once),
                },
                Mesh3d(obj_mesh.primitives[0].mesh.clone()),
                MeshMaterial3d(obj_mesh.primitives[0].material.clone().unwrap()),
                // Mesh3d(meshes.add(Sphere::new(1.0))),
                // MeshMaterial3d(materials.add(Color::WHITE)),
                RigidBody::Dynamic,
                Collider::ball(1.0),
                // ColliderConstructor::ConvexHullFromMesh,
                GravityScale(1.0),
                Transform::from_translation(
                    player_position_query.single().translation.clone() + Vec3::new(0.0, 2.0, 0.0),
                )
                .with_scale(Vec3::new(0.2, 0.2, 0.2)),
                Velocity {
                    linvel: camera_transform.translation,
                    angvel: Vec3::ZERO,
                },
                // CollisionGroups::new(collider_ball, collider_ground),
            ));
        }
    }

    print_timer_query.single_mut().0.tick(time.delta());
    if print_timer_query.single_mut().0.finished() {
        // println!("bullet number: {}", bullet_query.iter().len());
        print_timer_query.single_mut().0.reset();
    }
}

pub fn handle_keyboard_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_look_at: Res<CameraLookAt>,
    time: Res<Time>,
    mut controller_query: Query<&mut KinematicCharacterController, With<Player>>,
    mut player_velocity: Query<&mut Velocity, With<Player>>,
) {
    let mut look_direction = camera_look_at.look_at.normalize();
    look_direction.y = 0.0;
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

    // 角色位移
    let mut direc = direction.normalize_or_zero() * 20.0 * time.delta_secs();

    // 跳跃
    if keyboard.pressed(KeyCode::Space) {
        player_velocity.single_mut().linvel.y = 15.0;
    }

    if let Ok(mut controller) = controller_query.get_single_mut() {
        controller.translation = Some(direc);
    };
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

pub fn del_bullet(
    time: Res<Time>,
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut Bullet), With<Bullet>>,
) {
    for (entity, mut bullet) in bullets.iter_mut() {
        bullet.live_time.tick(time.delta());
        if bullet.live_time.finished() {
            commands.entity(entity).despawn();
        }
    }
}
