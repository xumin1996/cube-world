use bevy::input::mouse::MouseMotion;
use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use bevy_obj::ObjPlugin;
use core::f32;
use rand::Rng;
use bevy_demo::util::Triangle;


fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ObjPlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, handle_mouse_motion)
        .run();
}

pub fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // 平面
    let plane = Plane3d::default().mesh().size(1f32, 1f32).subdivisions(1);
    commands.spawn((
        Mesh3d(meshes.add(plane)),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));

    // 正方体
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.2))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // 随机草地
    let mut rng = rand::thread_rng();
    let offsets: Vec<Transform> = (0..10000)
        .map(|i| {
            let offset_x = rng.gen_range(-0.5..0.5);
            let offset_z = rng.gen_range(-0.5..0.5);
            let rotate = rng.gen_range(0.0..f32::consts::TAU);
            Transform::from_xyz(offset_x, 0.0, offset_z)
                .with_rotation(Quat::from_rotation_y(rotate))
        })
        .collect();
    let ts = Triangle::pieces(5) * offsets;
    commands.spawn((
        Mesh3d(meshes.add(ts.build())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            // unlit: true,
            ..default()
        })),
    ));

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(1.5, 1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn handle_mouse_motion(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera_transform: Query<&mut Transform, With<Camera>>,
) {
    let displacement = mouse_motion_events
        .read()
        .fold(0., |acc, mouse_motion| acc + mouse_motion.delta.x);

    // 旋转
    camera_transform
        .single_mut()
        .rotate_around(Vec3::ZERO, Quat::from_rotation_y(-displacement / 700.));
}
