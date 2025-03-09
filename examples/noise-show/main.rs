use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::render::{
    mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology,
};
use simdnoise::*;
use smooth_bevy_cameras::{LookTransform, Smoother};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, (handle_mouse_motion, handle_keyboard_controls))
        .run();
}

#[derive(Resource)]
pub struct OffsetXZ {
    x: i32,
    z: i32,
}

#[derive(Component, Debug)]
pub struct ViewRegion;

pub fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 平面
    let plain_height = get_mesh(0, 0);
    println!("plain_height: {:?}", plain_height);
    commands.spawn((
        ViewRegion,
        Mesh3d(meshes.add(plain_height)),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));

    // light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            color: Color::srgb(1.0, 1.0, 0.863),
            illuminance: 5000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 3.0)),
    ));

    // camera
    commands.spawn((
        LookTransform::new(Vec3::ZERO, Vec3::ZERO, Vec3::Y),
        Smoother::new(0.9),
        Camera3d::default(),
        Transform::from_xyz(30.0, 20.0, 0.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));

    commands.insert_resource(OffsetXZ { x: 0, z: 0 });
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

pub fn handle_keyboard_controls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    view_region_entity: Query<(Entity, &ViewRegion), With<ViewRegion>>,
    mut offsetXZ: ResMut<OffsetXZ>,
) {
    if keyboard.pressed(KeyCode::KeyW) {
        offsetXZ.z = offsetXZ.z - 1;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        offsetXZ.z = offsetXZ.z + 1;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        offsetXZ.x = offsetXZ.x - 1;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        offsetXZ.x = offsetXZ.x + 1;
    }

    println!("offsetXZ {} {}", offsetXZ.x, offsetXZ.z);

    for (entity, view_region) in view_region_entity.iter() {
        commands.entity(entity).despawn();
    }

    let plain_height = get_mesh(offsetXZ.x, offsetXZ.z);
    commands.spawn((
        ViewRegion,
        Mesh3d(meshes.add(plain_height)),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));
}

fn get_mesh(region_x: i32, region_z: i32) -> Mesh {
    let plain_size: u32 = 256u32;
    let heights = NoiseBuilder::fbm_2d_offset(
        region_x as f32,
        (plain_size + 1) as usize,
        region_z as f32,
        (plain_size + 1) as usize,
    )
    .with_seed(1)
    .generate_scaled(0.0, 10.0);
    let mut plain_height: Vec<Vec<f32>> = heights
        .chunks((plain_size + 1) as usize)
        .map(|chunk| {
            let mut rv = chunk.to_vec();
            rv
        })
        .collect();
    let collider_cube_mesh =
        create_plain_mesh(plain_size, &plain_height, Transform::from_xyz(-8f32, 0f32, -8f32));
    collider_cube_mesh
}

// 创建平面网格 (16+1)*(16+1) x,y
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
