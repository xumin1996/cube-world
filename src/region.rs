use std::time::Instant;

use crate::util::Triangle;
use crate::{customMaterial::CustomMaterial, player::Player};
use bevy::gltf::{Gltf, GltfMesh, GltfNode};
use bevy::prelude::*;
use bevy::render::{
    mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology,
};
use bevy::time::Stopwatch;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::RigidBodyActivation;
use simdnoise::*;

#[derive(Component, Debug)]
pub struct ViewRegion {
    block_x: i32,
    block_y: i32,
    block_z: i32,
}

#[derive(Component, Debug)]
pub struct RigidRegion {
    block_x: i32,
    block_y: i32,
    block_z: i32,
}

#[derive(Resource)]
pub struct MyAssetPacket(Handle<Gltf>);

const collider_player: Group = Group::GROUP_1;
const collider_ground: Group = Group::GROUP_2;
const collider_ball: Group = Group::GROUP_3;

pub fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 加载 .glb 文件
    let aio_handle = asset_server.load("models/aio.gltf");
    commands.insert_resource(MyAssetPacket(aio_handle));
}

pub fn region_update(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_position_query: Query<&Transform, With<Player>>,
    view_region_entity: Query<(Entity, &ViewRegion), With<ViewRegion>>,
    rigid_region_entity: Query<(Entity, &RigidRegion), With<RigidRegion>>,
    my_asset_packet: Res<MyAssetPacket>,
    gltf_asset: Res<Assets<Gltf>>,
    gltf_node_asset: Res<Assets<GltfNode>>,
    gltf_mesh_asset: Res<Assets<GltfMesh>>,
) {
    let view_circle = 8;
    let rigid_circle = 3;
    // 角色所在区块
    let player_region_x = player_position_query.single().translation.x as i32 / 16;
    let player_region_y = player_position_query.single().translation.y as i32 / 16;
    let player_region_z = player_position_query.single().translation.z as i32 / 16;

    // 删除已有区块
    let mut view_region_list: Vec<&ViewRegion> = Vec::new();
    for (entity, view_region) in view_region_entity.iter() {
        view_region_list.push(&view_region);
        if !in_region(
            view_region.block_x,
            view_region.block_y,
            view_region.block_z,
            player_region_x,
            player_region_y,
            player_region_z,
            view_circle,
        ) {
            commands.entity(entity).despawn();
        }
    }
    let mut rigid_region_list: Vec<&RigidRegion> = Vec::new();
    for (entity, rigid_region) in rigid_region_entity.iter() {
        rigid_region_list.push(&rigid_region);
        if !in_region(
            rigid_region.block_x,
            rigid_region.block_y,
            rigid_region.block_z,
            player_region_x,
            player_region_y,
            player_region_z,
            rigid_circle,
        ) {
            commands.entity(entity).despawn();
        }
    }

    // view地形 默认加载周围(view_circle * view_circle)的区块
    let cube_material = materials.add(Color::WHITE);
    for region_x in player_region_x - view_circle..=player_region_x + view_circle {
        for region_z in player_region_z - view_circle..=player_region_z + view_circle {
            // 检查是否已经存在
            let fit_num = view_region_list
                .iter()
                .filter(|v| v.block_x == region_x && v.block_z == region_z)
                .count();

            if fit_num == 0 {
                let plain_height = get_mesh(region_x, region_z);
                commands.spawn((
                    ViewRegion {
                        block_x: region_x,
                        block_y: 0,
                        block_z: region_z,
                    },
                    Mesh3d(meshes.add(plain_height)),
                    MeshMaterial3d(cube_material.clone()),
                ));
            }
        }
    }

    // rigid地形 加载周围(rigid_circle * rigid_circle)的区块
    for region_x in player_region_x - rigid_circle..=player_region_x + rigid_circle {
        for region_z in player_region_z - rigid_circle..=player_region_z + rigid_circle {
            // 检查是否存在
            let fit_num = rigid_region_list
                .iter()
                .filter(|v| v.block_x == region_x && v.block_z == region_z)
                .count();
            if fit_num == 0 {
                let start = Instant::now();
                let plain_mesh = get_mesh(region_x, region_z);
                println!("get_mesh time: {}", (Instant::now() - start).as_secs_f32());

                let plain_tri = Triangle::from_mesh(&plain_mesh);
                let plain_indices = (0..plain_tri.points.len())
                    .into_iter()
                    .map(|n| n as u32)
                    .collect::<Vec<u32>>()
                    .chunks(3)
                    .map(|vs| [vs[0], vs[1], vs[2]])
                    .collect();
                println!("plain_tri time: {}", (Instant::now() - start).as_secs_f32());

                let start = Instant::now();
                let trimesh = Collider::trimesh(plain_tri.points, plain_indices);
                println!("trimesh time: {}", (Instant::now() - start).as_secs_f32());
                
                let start = Instant::now();
                commands.spawn((
                    RigidRegion {
                        block_x: region_x,
                        block_y: 0,
                        block_z: region_z,
                    },
                    RigidBody::Fixed,
                    trimesh,
                    // CollisionGroups::new(collider_ground, collider_player | collider_ball ),
                ));
                println!("spawn time: {}", (Instant::now() - start).as_secs_f32());
            }
        }
    }
}

fn in_region(bx: i32, by: i32, bz: i32, px: i32, py: i32, pz: i32, region: i32) -> bool {
    if (bx - px).abs() <= region && (bz - pz).abs() <= region {
        return true;
    }
    return false;
}

fn get_map_height(region_x: i32, region_z: i32) -> Vec<Vec<f32>> {
    let plain_size = 16i32;
    // [x1,x1,x1,...,x2,x2,x2,...,x3,x3,x3,....xy, xy,xy,...]
    let (heights, min, max) = NoiseBuilder::fbm_2d_offset(
        (region_z * plain_size) as f32,
        (plain_size + 1) as usize,
        (region_x * plain_size) as f32,
        (plain_size + 1) as usize,
    )
    .with_seed(1)
    .generate();
    let heights: Vec<f32> = heights.iter().map(|item| (item * 100f32).floor()).collect();
    let plain_height: Vec<Vec<f32>> = heights
        .chunks((plain_size + 1) as usize)
        .map(|chunk| {
            let mut rv = chunk.to_vec();
            rv
        })
        .collect();
    return plain_height;
}

fn get_mesh(region_x: i32, region_z: i32) -> Mesh {
    let plain_size = 16i32;
    let plain_height: Vec<Vec<f32>> = get_map_height(region_x, region_z);

    let start = Instant::now();
    let collider_cube_mesh = create_cube_mesh(
        &plain_height,
        Transform::from_xyz(
            region_x as f32 * plain_size as f32,
            0f32,
            region_z as f32 * plain_size as f32,
        ),
    );
    println!(
        "create mesh time: {}",
        (Instant::now() - start).as_secs_f32()
    );
    collider_cube_mesh
}

// 创建平面网格 (16+1)*(16+1) x,y
fn create_plain_mesh(height_mesh: &Vec<Vec<f32>>, transform: Transform) -> Mesh {
    let plain_size = 16i32;
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
    for x in 0..16 {
        for z in 0..16 {
            let start_index: u32 = x * 17 + z;
            let short_indices: Vec<u32> = vec![0, 1, 17, 1, 18, 17]
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
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, attribute_normal)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, attribute_uv_0)
    .with_inserted_indices(Indices::U32(indices))
}

fn create_cube_mesh(height_mesh: &Vec<Vec<f32>>, transform: Transform) -> Mesh {
    let plain_size = 16usize;
    let mut cube_transform = Vec::<Transform>::new();

    for (x_index, z_list) in height_mesh.iter().take(plain_size).enumerate() {
        for (z_index, y_height) in z_list.iter().take(plain_size).enumerate() {
            // 顶点
            let cube_size = 1f32;
            let x = cube_size * x_index as f32 + transform.translation.x;
            let y = *y_height;
            let z = cube_size * z_index as f32 + transform.translation.z;
            cube_transform.push(Transform::from_xyz(x, y, z));
        }
    }

    return create_cubes(&cube_transform);
}

// 构造Mesh
fn create_cubes(cube_positions: &Vec<Transform>) -> Mesh {
    let mut attribute_position: Vec<[f32; 3]> = Vec::new();
    let mut attribute_uv_0: Vec<[f32; 2]> = Vec::new();
    let mut attribute_normal: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for cube_position in cube_positions.iter() {
        let pre_attribute_position_num = attribute_position.len();
        let mut item_position = vec![
            // top (facing towards +y)
            [-0.5, 0.5, -0.5], // vertex with index 0
            [0.5, 0.5, -0.5],  // vertex with index 1
            [0.5, 0.5, 0.5],   // etc. until 23
            [-0.5, 0.5, 0.5],
            // bottom   (-y)
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [-0.5, -0.5, 0.5],
            // right    (+x)
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5], // This vertex is at the same position as vertex with index 2, but they'll have different UV and normal
            [0.5, 0.5, -0.5],
            // left     (-x)
            [-0.5, -0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [-0.5, 0.5, -0.5],
            // back     (+z)
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [0.5, 0.5, 0.5],
            [0.5, -0.5, 0.5],
            // forward  (-z)
            [-0.5, -0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [0.5, 0.5, -0.5],
            [0.5, -0.5, -0.5],
        ];
        for mut position_item in item_position.iter_mut() {
            position_item[0] += cube_position.translation.x;
            position_item[1] += cube_position.translation.y;
            position_item[2] += cube_position.translation.z;
        }

        attribute_position.extend(item_position);
        attribute_uv_0.extend(vec![
            // Assigning the UV coords for the top side.
            [0.0, 0.2],
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 0.2],
            // Assigning the UV coords for the bottom side.
            [0.0, 0.45],
            [0.0, 0.25],
            [1.0, 0.25],
            [1.0, 0.45],
            // Assigning the UV coords for the right side.
            [1.0, 0.45],
            [0.0, 0.45],
            [0.0, 0.2],
            [1.0, 0.2],
            // Assigning the UV coords for the left side.
            [1.0, 0.45],
            [0.0, 0.45],
            [0.0, 0.2],
            [1.0, 0.2],
            // Assigning the UV coords for the back side.
            [0.0, 0.45],
            [0.0, 0.2],
            [1.0, 0.2],
            [1.0, 0.45],
            // Assigning the UV coords for the forward side.
            [0.0, 0.45],
            [0.0, 0.2],
            [1.0, 0.2],
            [1.0, 0.45],
        ]);
        attribute_normal.extend(vec![
            // Normals for the top side (towards +y)
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            // Normals for the bottom side (towards -y)
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            // Normals for the right side (towards +x)
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            // Normals for the left side (towards -x)
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            // Normals for the back side (towards +z)
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            // Normals for the forward side (towards -z)
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
        ]);

        let indices_temp: Vec<u32> = vec![
            0, 3, 1, 1, 3, 2, // triangles making up the top (+y) facing side.
            4, 5, 7, 5, 6, 7, // bottom (-y)
            8, 11, 9, 9, 11, 10, // right (+x)
            12, 13, 15, 13, 14, 15, // left (-x)
            16, 19, 17, 17, 19, 18, // back (+z)
            20, 21, 23, 21, 22, 23, // forward (-z)
        ];
        let indices_maped: Vec<u32> = indices_temp
            .iter()
            .map(|item| item + pre_attribute_position_num as u32)
            .collect();
        indices.extend(indices_maped);
    }

    // Keep the mesh data accessible in future frames to be able to mutate it in toggle_texture.
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, attribute_position)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, attribute_uv_0)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, attribute_normal)
    .with_inserted_indices(Indices::U32(indices))
}
