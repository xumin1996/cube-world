use std::f32::consts::PI;

use bevy::gltf::{Gltf, GltfMesh, GltfNode};
use bevy::input::mouse::MouseMotion;
use bevy::math::VectorSpace;
use bevy::prelude::*;
use bevy::render::render_resource::Texture;
use bevy::state::commands;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use smooth_bevy_cameras::{LookTransform, Smoother};

#[derive(Component)]
pub struct Npc {
    towards: Vec3,
}

// 定义按键冷却时间的组件
#[derive(Component)]
pub struct RotateTimer(Timer);

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
                min_width: CharacterLength::Absolute(0.01),
                include_dynamic_bodies: true,
            }),
            ..default()
        },
        LockedAxes::ROTATION_LOCKED,
        RigidBody::Dynamic,
        Collider::ball(0.5),
        GravityScale(4.0),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::from(Srgba::BLUE))),
        // SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/Fox.glb"))),
        Transform::from_xyz(0.0, 10.0, 0.0),
        Npc {
            towards: Vec3::ZERO,
        },
        Ccd::enabled(),
        Velocity::zero(),
        // CollisionGroups::new(collider_player, collider_ground),
    ));

    // 转向冷却时间
    commands.spawn(RotateTimer(Timer::from_seconds(3.0, TimerMode::Once)));
}

pub fn handle_keyboard_controls(
    time: Res<Time>,
    mut rotate_timer_query: Query<&mut RotateTimer>,
    mut controller_query: Query<(&mut KinematicCharacterController, &mut Npc), With<Npc>>,
) {
    if let Ok(mut rotate_timer) = rotate_timer_query.single_mut() {
        rotate_timer.0.tick(time.delta());
        if let Ok((mut controller, mut npc)) = controller_query.single_mut() {
            if rotate_timer.0.finished() {
                rotate_timer.0.reset();

                // 确定随机方向
                let mut rng = rand::thread_rng();
                let rotation_quaternion =
                    Quat::from_rotation_y(rng.gen::<f32>() * std::f32::consts::TAU);
                let direction = rotation_quaternion.mul_vec3(Vec3::X.normalize() * 2.0);

                // 改变方向
                println!("direction:{}", direction);
                npc.towards = direction;
            }

            // 移动
            controller.translation = Some(npc.towards * time.delta_secs());
        };
    }
}
