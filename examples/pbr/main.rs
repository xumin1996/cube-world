use bevy::asset::RenderAssetUsages;
use bevy::image::ImageLoaderSettings;
use bevy::input::mouse::MouseMotion;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use std::f32::consts::FRAC_PI_2;

#[derive(Component, Debug)]
pub struct Cube;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
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
    // 加载 PBR 贴图
    let base_color_texture: Handle<Image> = asset_server.load("textures/cobblestone.png");
    let normal_texture: Handle<Image> = asset_server.load("textures/cobblestone_n.png");
    let metallic_roughness_texture: Handle<Image> =
        asset_server.load("textures/cobblestone_s.png");

    // 创建 PBR 材质
    let material: Handle<StandardMaterial> = materials.add(StandardMaterial {
        base_color_texture: Some(base_color_texture),
        normal_map_texture: Some(normal_texture),
        metallic_roughness_texture: Some(metallic_roughness_texture),
        ..default()
    });

    // 需要 法线贴图纹理,顶点 UV,顶点切线,顶点法线
    // https://docs.rs/bevy/latest/bevy/pbr/struct.StandardMaterial.html#structfield.normal_map_texture
    let mut base_mesh = Cuboid::new(1.0, 1.0, 1.0).mesh().build();
    base_mesh
        .generate_tangents()
        .expect("generate tangents fail");
    commands.spawn((
        Mesh3d(meshes.add(base_mesh)), 
        MeshMaterial3d(material),
        Cube,
    ));

    // 环境光
    commands.insert_resource(AmbientLight {
        brightness: 1000.0,
        color: Color::srgb(1.0, 1.0, 1.0),
        ..default()
    });

    // 平行光
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            color: Color::srgb(1.0, 1.0, 0.863),
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 8.0)),
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(3.0, 1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn handle_mouse_motion(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut cube_transform: Query<&mut Transform, With<Cube>>,
) {
    let displacement = mouse_motion_events
        .read()
        .fold(0., |acc, mouse_motion| acc + mouse_motion.delta.x);

    // 旋转
    cube_transform
        .single_mut()
        .rotate_around(Vec3::ZERO, Quat::from_rotation_y(displacement / 700.));
}
