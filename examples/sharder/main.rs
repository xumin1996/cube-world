use bevy::input::mouse::MouseMotion;
use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MaterialPlugin::<CustomMaterial>::default()))
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
    commands.spawn((MaterialMeshBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(CustomMaterial {}),
        ..default()
    },));

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(3.0, 2.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
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

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {}

impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/animate_shader.wgsl".into()
    }
}
