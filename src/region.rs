use super::player::Player;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::render::{
    mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology,
};
use noise::{NoiseFn, Perlin};

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

pub fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 角色所在区块
    let player_region_x = 0i32;
    let player_region_y = 0i32;
    let player_region_z = 0i32;
    println!(
        "player region, x {}, y:{}, z: {}",
        player_region_x, player_region_y, player_region_z
    );

    // view地形 默认加载周围25(5*5)个区块
    let cube_material = materials.add(Color::WHITE);
    for region_x in player_region_x - 2..=player_region_x + 2 {
        for region_z in player_region_z - 2..=player_region_z + 2 {
            println!("setup add view x: {}, z: {}", region_x, region_z);
            let mut cube_positions_x: Vec<Vec<f32>> = Vec::new();
            for region_block_x in 0..17 {
                let mut cube_positions_z: Vec<f32> = Vec::new();
                for region_block_z in 0..17 {
                    let block_x = region_x * 16 + region_block_x;
                    let block_z = region_z * 16 + region_block_z;
                    let height = get_height(block_x, 0, block_z);
                    cube_positions_z.push(height);
                }
                cube_positions_x.push(cube_positions_z);
            }
            commands.spawn((
                ViewRegion {
                    block_x: region_x,
                    block_y: 0,
                    block_z: region_z,
                },
                PbrBundle {
                    mesh: meshes.add(create_plain_mesh(
                        &cube_positions_x,
                        Transform::from_xyz(region_x as f32 * 16f32, 0f32, region_z as f32 * 16f32),
                    )),
                    material: cube_material.clone(),
                    ..default()
                },
            ));
        }
    }

    // rigid地形 加载周围9(3*3)个区块
    for region_x in player_region_x - 1..=player_region_x + 1 {
        for region_z in player_region_z - 1..=player_region_z + 1 {
            println!("setup add rigid x: {}, z: {}", region_x, region_z);
            let mut cube_positions_x: Vec<Vec<f32>> = Vec::new();
            for region_block_x in 0..17 {
                let mut cube_positions_z: Vec<f32> = Vec::new();
                for region_block_z in 0..17 {
                    let block_x = region_x * 16 + region_block_x;
                    let block_z = region_z * 16 + region_block_z;
                    let height = get_height(block_x, 0, block_z);
                    cube_positions_z.push(height);
                }
                cube_positions_x.push(cube_positions_z);
            }
            let collider_cube_mesh = create_plain_mesh(
                &cube_positions_x,
                Transform::from_xyz(region_x as f32 * 16f32, 0f32, region_z as f32 * 16f32),
            );
            commands.spawn((
                RigidRegion {
                    block_x: region_x,
                    block_y: 0,
                    block_z: region_z,
                },
                RigidBody::Static,
                Collider::trimesh_from_mesh(&collider_cube_mesh).unwrap(),
            ));
        }
    }
}

pub fn region_update(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_position_query: Query<&Transform, With<Player>>,
    view_region_entity: Query<(Entity, &ViewRegion), With<ViewRegion>>,
    rigid_region_entity: Query<(Entity, &RigidRegion), With<RigidRegion>>,
) {
    let view_circle = 8;
    let rigid_circle = 1;
    // 角色所在区块
    let player_region_x = player_position_query.single().translation.x as i32 / 16;
    let player_region_y = player_position_query.single().translation.y as i32 / 16;
    let player_region_z = player_position_query.single().translation.z as i32 / 16;
    // println!(
    //     "player region, x {}, y:{}, z: {}",
    //     player_region_x, player_region_y, player_region_z
    // );

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
            println!("delete {:?}", view_region);
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
            println!("delete {:?}", rigid_region);
        }
    }

    // view地形 默认加载周围25(5*5)个区块
    let cube_material = materials.add(Color::WHITE);
    for region_x in player_region_x - view_circle..=player_region_x + view_circle {
        for region_z in player_region_z - view_circle..=player_region_z + view_circle {
            // 检查是否已经存在
            let fit_num = view_region_list
                .iter()
                .filter(|v| v.block_x == region_x && v.block_z == region_z)
                .count();

            if fit_num == 0 {
                println!("update add view x: {}, z: {}", region_x, region_z);
                let mut cube_positions_x: Vec<Vec<f32>> = Vec::new();
                for region_block_x in 0..17 {
                    let mut cube_positions_z: Vec<f32> = Vec::new();
                    for region_block_z in 0..17 {
                        let block_x = region_x * 16 + region_block_x;
                        let block_z = region_z * 16 + region_block_z;
                        let height = get_height(block_x, 0, block_z);
                        cube_positions_z.push(height);
                    }
                    cube_positions_x.push(cube_positions_z);
                }
                commands.spawn((
                    ViewRegion {
                        block_x: region_x,
                        block_y: 0,
                        block_z: region_z,
                    },
                    PbrBundle {
                        mesh: meshes.add(create_plain_mesh(
                            &cube_positions_x,
                            Transform::from_xyz(
                                region_x as f32 * 16f32,
                                0f32,
                                region_z as f32 * 16f32,
                            ),
                        )),
                        material: cube_material.clone(),
                        ..default()
                    },
                ));
            }
        }
    }

    // rigid地形 加载周围9(3*3)个区块
    for region_x in player_region_x - rigid_circle..=player_region_x + rigid_circle {
        for region_z in player_region_z - rigid_circle..=player_region_z + rigid_circle {
            // 检查是否存在
            let fit_num = rigid_region_list
                .iter()
                .filter(|v| v.block_x == region_x && v.block_z == region_z)
                .count();
            if fit_num == 0 {
                println!("update add rigid x: {}, z: {}", region_x, region_z);
                let mut cube_positions_x: Vec<Vec<f32>> = Vec::new();
                for region_block_x in 0..17 {
                    let mut cube_positions_z: Vec<f32> = Vec::new();
                    for region_block_z in 0..17 {
                        let block_x = region_x * 16 + region_block_x;
                        let block_z = region_z * 16 + region_block_z;
                        let height = get_height(block_x, 0, block_z);
                        cube_positions_z.push(height);
                    }
                    cube_positions_x.push(cube_positions_z);
                }
                let collider_cube_mesh = create_plain_mesh(
                    &cube_positions_x,
                    Transform::from_xyz(region_x as f32 * 16f32, 0f32, region_z as f32 * 16f32),
                );
                commands.spawn((
                    RigidRegion {
                        block_x: region_x,
                        block_y: 0,
                        block_z: region_z,
                    },
                    RigidBody::Static,
                    Collider::trimesh_from_mesh(&collider_cube_mesh).unwrap(),
                ));
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

fn get_height(x: i32, y: i32, z: i32) -> f32 {
    let region_max_size = 128f64;
    let perlin = Perlin::new(1);
    let height = perlin.get([x as f64 / region_max_size, z as f64 / region_max_size]);
    return (height as f32 * 20.0f32).round();
}

// 创建平面网格 (16+1)*(16+1) x,y
fn create_plain_mesh(height_mesh: &Vec<Vec<f32>>, transform: Transform) -> Mesh {
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
            let uv_size = 1f32 / 16f32;
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
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, attribute_uv_0)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, attribute_normal)
    .with_inserted_indices(Indices::U32(indices))
}
