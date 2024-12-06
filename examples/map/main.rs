use bevy::input::mouse::MouseMotion;
use bevy::render::{
    mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology,
};
use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use noise::{NoiseFn, Perlin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins))
        .add_systems(Startup, startup)
        .add_systems(Update, handle_mouse_motion)
        .run();
}

pub fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 平面
    let plain_size = 128u32;
    let mut plain_height: Vec<Vec<f32>> = Vec::new();
    for plain_x in 0..=plain_size {
        let mut z_height: Vec<f32> = Vec::new();
        for plain_z in 0..=plain_size {
            let block_x = plain_x as f64 / plain_size as f64;
            let block_z = plain_z as f64 / plain_size as f64;
            let height = get_height(block_x, block_z);
            z_height.push(height);
        }
        plain_height.push(z_height);
    }
    commands.spawn(PbrBundle {
        mesh: meshes.add(create_plain_mesh(
            plain_size,
            &plain_height,
            Transform::from_xyz(0f32, 0f32, 0f32),
        )),
        material: materials.add(Color::WHITE),
        ..default()
    });

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

fn get_height(x: f64, z: f64) -> f32 {
    let perlin = Perlin::new(1);
    let height = perlin.get([x, z]);
    return height as f32 * 500.0f32;
}

// 创建平面网格 (plain_size+1)*(plain_size+1) x,y
fn create_plain_mesh(plain_size: u32, height_mesh: &Vec<Vec<f32>>, transform: Transform) -> Mesh {
    let mut attribute_position: Vec<[f32; 3]> = Vec::new();
    let mut attribute_uv_0: Vec<[f32; 2]> = Vec::new();
    let mut attribute_normal: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for (x_index, z_list) in height_mesh.iter().enumerate() {
        for (z_index, y_height) in z_list.iter().enumerate() {
            // 顶点
            let cube_size = 1f32;
            let x = cube_size * x_index as f32 + transform.translation.x;
            let y = *y_height;
            let z = cube_size * z_index as f32 + transform.translation.z;
            attribute_position.push([x, y, z]);

            // uv
            let uv_size = 1f32 / plain_size as f32;
            attribute_uv_0.push([uv_size * x_index as f32, uv_size * z_index as f32]);

            // 法线
            attribute_normal.push([0.0, 1.0, 0.0]);
        }
    }

    // 索引
    for x in 0..plain_size {
        for z in 0..plain_size {
            let start_index: u32 = x * (plain_size + 1) + z;
            let short_indices: Vec<u32> =
                vec![0, 1, plain_size + 1, 1, plain_size + 2, plain_size + 1]
                    .iter()
                    .map(|index| index + start_index)
                    .collect();
            indices.extend(short_indices);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, attribute_position)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, attribute_uv_0)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, attribute_normal)
    .with_inserted_indices(Indices::U32(indices))
}
