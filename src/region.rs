use std::time::Instant;

use crate::player::Player;
use crate::util::Triangle;
use bevy::gltf::{Gltf, GltfMesh, GltfNode};
use bevy::prelude::*;
use bevy::render::{
    mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology,
};
use bevy_rapier3d::prelude::*;
use rand::Rng;
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
pub struct LowPolySanBlockAsset(Handle<Gltf>);

const collider_player: Group = Group::GROUP_1;
const collider_ground: Group = Group::GROUP_2;
const collider_ball: Group = Group::GROUP_3;

pub fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 加载 .glb 文件
    let low_poly_sand_block_handle = asset_server.load("models/stylized_low-poly_sand_block.glb");
    commands.insert_resource(LowPolySanBlockAsset(low_poly_sand_block_handle));

    // 环境光
    commands.insert_resource(AmbientLight {
        brightness: 1000.0,
        color: Color::srgb(0.2, 0.2, 0.2),
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
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 3.0)),
    ));
}

pub fn region_update(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_position_query: Query<&Transform, With<Player>>,
    view_region_entity: Query<(Entity, &ViewRegion), With<ViewRegion>>,
    rigid_region_entity: Query<(Entity, &RigidRegion), With<RigidRegion>>,
    sand_block_query: Res<LowPolySanBlockAsset>,
    gltf_asset: Res<Assets<Gltf>>,
    gltf_node_asset: Res<Assets<GltfNode>>,
    gltf_mesh_asset: Res<Assets<GltfMesh>>,
    asset_server: Res<AssetServer>,
) {
    let view_circle = 10;
    let rigid_circle = 5;
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
    if let Some(obj_mesh) = gltf_asset
        .get(&sand_block_query.0)
        .and_then(|gltf| gltf_node_asset.get(&gltf.named_nodes["block"]))
        .and_then(|floor_dirt| floor_dirt.mesh.as_ref())
        .and_then(|floor_mesh_handle| gltf_mesh_asset.get(floor_mesh_handle))
    {
        // 加载 PBR 贴图
        let base_color_texture: Handle<Image> = asset_server.load("textures/grass_block_top.png");
        let normal_texture: Handle<Image> = asset_server.load("textures/grass_block_top_n.png");
        let metallic_roughness_texture: Handle<Image> =
            asset_server.load("textures/grass_block_top_mr.png");

        // 创建 PBR 材质
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.5647, 0.8039, 0.5922),
            base_color_texture: Some(base_color_texture),
            normal_map_texture: Some(normal_texture),
            metallic: 1.0,
            perceptual_roughness: 1.0,
            metallic_roughness_texture: Some(metallic_roughness_texture),
            ..default()
        });

        // let cube_material = materials.add(Color::WHITE);
        for region_x in player_region_x - view_circle..=player_region_x + view_circle {
            for region_z in player_region_z - view_circle..=player_region_z + view_circle {
                // 检查是否已经存在
                let fit_num = view_region_list
                    .iter()
                    .filter(|v| v.block_x == region_x && v.block_z == region_z)
                    .count();

                if fit_num == 0 {
                    let region_mesh: Mesh = region_by_block(region_x, region_z);

                    // 区块偏移
                    let plain_size = 16i32;
                    let region_transform = Transform::from_xyz(
                        region_x as f32 * plain_size as f32,
                        0f32,
                        region_z as f32 * plain_size as f32,
                    );

                    // let block_mesh: &Mesh = meshes.get(&obj_mesh.primitives[0].mesh).unwrap();
                    // let region_mesh: Mesh = region_by_mesh(region_x, region_z, block_mesh);
                    commands.spawn((
                        ViewRegion {
                            block_x: region_x,
                            block_y: 0,
                            block_z: region_z,
                        },
                        // Mesh3d(meshes.add(region_mesh)),
                        // MeshMaterial3d(obj_mesh.primitives[0].material.clone().unwrap()),
                        Mesh3d(meshes.add(region_mesh)),
                        // MeshMaterial3d(cube_material.clone()),
                        MeshMaterial3d(material.clone()),
                        region_transform,
                    ));
                }
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
                let plain_mesh = region_by_block(region_x, region_z);
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

                // 区块偏移
                let plain_size = 16i32;
                let region_transform = Transform::from_xyz(
                    region_x as f32 * plain_size as f32,
                    0f32,
                    region_z as f32 * plain_size as f32,
                );

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
                    region_transform,
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

fn region_by_block(region_x: i32, region_z: i32) -> Mesh {
    let plain_height: Vec<Vec<f32>> = get_map_height(region_x, region_z);

    let start = Instant::now();
    let mut collider_cube_mesh = create_cube_mesh(&plain_height);
    collider_cube_mesh
        .generate_tangents()
        .expect("generate_tangents fail");
    println!(
        "create mesh time: {}",
        (Instant::now() - start).as_secs_f32()
    );
    collider_cube_mesh
}

fn region_by_mesh(region_x: i32, region_z: i32, mesh_obj: &Mesh) -> Mesh {
    let plain_size = 16usize;
    let plain_height: Vec<Vec<f32>> = get_map_height(region_x, region_z);

    let start = Instant::now();
    let mut block_tris = Vec::<Transform>::new();
    for (x_index, z_list) in plain_height.iter().take(plain_size).enumerate() {
        for (z_index, y_height) in z_list.iter().take(plain_size).enumerate() {
            // 顶点
            let cube_size = 1f32;
            let x = cube_size * x_index as f32;
            let y = *y_height;
            let z = cube_size * z_index as f32;
            let block_transform = Transform::from_xyz(x, y, z).with_scale(Vec3::new(1.3, 1.3, 1.3));
            block_tris.push(block_transform);
        }
    }
    let block_tri: Triangle = Triangle::from_mesh(mesh_obj);
    let block_tri = block_tri * block_tris;
    println!(
        "region_by_mesh mesh time: {}",
        (Instant::now() - start).as_secs_f32()
    );
    block_tri.build()
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

fn create_cube_mesh(height_mesh: &Vec<Vec<f32>>) -> Mesh {
    let plain_size = 16usize;
    let mut cube_transform: Vec<Transform> = Vec::<Transform>::new();

    for (x_index, z_list) in height_mesh.iter().take(plain_size).enumerate() {
        for (z_index, y_height) in z_list.iter().take(plain_size).enumerate() {
            // 顶点
            let cube_size = 1f32;
            let x = cube_size * x_index as f32;
            let y = *y_height;
            let z = cube_size * z_index as f32;
            cube_transform.push(Transform::from_xyz(x, y, z));
        }
    }

    let mut rng = rand::thread_rng();
    let x_off = rng.gen_range(0..2) as f32 * (1.0 / 2.0);
    let y_off = rng.gen_range(0..2) as f32 * (1.0 / 2.0);
    let mut cube_mesh: Triangle = Triangle::from_mesh(&Cuboid::new(1.0, 1.0, 1.0).mesh().build());
    cube_mesh.uv.iter_mut().for_each(|item|{
        item.x = item.x / 2.0 + x_off;
        item.y = item.y / 2.0 + y_off; 
    });
    let plain_mesh: Triangle = cube_mesh * cube_transform;

    plain_mesh.build()
}
