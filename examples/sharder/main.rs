use bevy::input::mouse::MouseMotion;
use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use std::f32::consts::FRAC_PI_2;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins,    MaterialPlugin::<CustomMaterial>::default()))
        .add_systems(Startup, startup)
        .add_systems(Update, handle_mouse_motion)
        .run();
}

pub fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    // 方块
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(CustomMaterial {})),
    ));

    // 平面
    // commands.spawn((
    //     Mesh3d(meshes.add(Plane3d::new(Vec3::new(0.0, 1.0, 0.0), Vec2::new(1.0, 1.0)))),
    //     MeshMaterial3d(materials.add(CustomMaterial {})),
    //     Transform::from_xyz(0.0, 0.5, 0.0).with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
    // ));

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
        Transform::from_xyz(3.0, 1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn handle_mouse_motion(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera_transform_query: Query<&mut Transform, With<Camera>>,
) {
    let displacement = mouse_motion_events
        .read()
        .fold(0., |acc, mouse_motion| acc + mouse_motion.delta.x);

    // 旋转
    if let Ok(mut camera_transform) = camera_transform_query.single_mut() {
        camera_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(-displacement / 700.));
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {}

impl Material for CustomMaterial {
    // fn vertex_shader() -> ShaderRef {
    //     "shaders/animate_shader.wgsl".into()
    // }
    fn fragment_shader() -> ShaderRef {
        "shaders/animate_shader.wgsl".into()
    }
}
