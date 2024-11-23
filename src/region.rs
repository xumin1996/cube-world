use super::player::Player;
use avian3d::{
    parry::{math, shape},
    prelude::*,
};
use bevy::render::{
    mesh::{Indices, VertexAttributeValues},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};
use bevy::{ecs::entity, prelude::*};
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
            let mut cube_positions: Vec<Transform> = Vec::new();
            for region_block_x in 0..16 {
                for region_block_z in 0..16 {
                    let block_x = region_x * 16 + region_block_x;
                    let block_z = region_z * 16 + region_block_z;
                    let height = get_height(block_x, 0, block_z);
                    cube_positions.push(Transform::from_xyz(
                        block_x as f32,
                        height,
                        block_z as f32,
                    ));
                }
            }
            commands.spawn((
                ViewRegion {
                    block_x: region_x,
                    block_y: 0,
                    block_z: region_z,
                },
                PbrBundle {
                    mesh: meshes.add(create_cube_mesh(&cube_positions)),
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
            let mut collider_cube_positions: Vec<Transform> = Vec::new();
            for region_block_x in 0..16 {
                for region_block_z in 0..16 {
                    let block_x = region_x * 16 + region_block_x;
                    let block_z = region_z * 16 + region_block_z;
                    let height = get_height(block_x, 0, block_z);
                    collider_cube_positions.push(Transform::from_xyz(
                        block_x as f32,
                        height,
                        block_z as f32,
                    ));
                }
            }
            let collider_cube_mesh = create_cube_mesh(&collider_cube_positions);
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
    let view_circle = 5;
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
        if (!in_region(
            view_region.block_x,
            view_region.block_y,
            view_region.block_z,
            player_region_x,
            player_region_y,
            player_region_z,
            view_circle,
        )) {
            commands.entity(entity).despawn();
            println!("delete {:?}", view_region);
        }
    }
    let mut rigid_region_list: Vec<&RigidRegion> = Vec::new();
    for (entity, rigid_region) in rigid_region_entity.iter() {
        rigid_region_list.push(&rigid_region);
        if (!in_region(
            rigid_region.block_x,
            rigid_region.block_y,
            rigid_region.block_z,
            player_region_x,
            player_region_y,
            player_region_z,
            rigid_circle,
        )) {
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
            if (fit_num == 0) {
                println!("update add view x: {}, z: {}", region_x, region_z);
                let mut cube_positions: Vec<Transform> = Vec::new();
                for region_block_x in 0..16 {
                    for region_block_z in 0..16 {
                        let block_x = region_x * 16 + region_block_x;
                        let block_z = region_z * 16 + region_block_z;
                        let height = get_height(block_x, 0, block_z);
                        cube_positions.push(Transform::from_xyz(
                            block_x as f32,
                            height,
                            block_z as f32,
                        ));
                    }
                }
                commands.spawn((
                    ViewRegion {
                        block_x: region_x,
                        block_y: 0,
                        block_z: region_z,
                    },
                    PbrBundle {
                        mesh: meshes.add(create_cube_mesh(&cube_positions)),
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
            if (fit_num == 0) {
                println!("update add rigid x: {}, z: {}", region_x, region_z);
                let mut collider_cube_positions: Vec<Transform> = Vec::new();
                for region_block_x in 0..16 {
                    for region_block_z in 0..16 {
                        let block_x = region_x * 16 + region_block_x;
                        let block_z = region_z * 16 + region_block_z;
                        let height = get_height(block_x, 0, block_z);
                        collider_cube_positions.push(Transform::from_xyz(
                            block_x as f32,
                            height,
                            block_z as f32,
                        ));
                    }
                }
                let collider_cube_mesh = create_cube_mesh(&collider_cube_positions);
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
    if ((bx - px).abs() <= region && (bz - pz).abs() <= region) {
        return true;
    }
    return false;
}

fn get_height(x: i32, y: i32, z: i32) -> f32 {
    let perlin = Perlin::new(1);
    let height = perlin.get([x as f64 / 100.0, z as f64 / 100.0]);
    return (height as f32 * 20.0f32).round();
}

// 构造Mesh
fn create_cube_mesh(cube_positions: &Vec<Transform>) -> Mesh {
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
    // Each array is an [x, y, z] coordinate in local space.
    // The camera coordinate space is right-handed x-right, y-up, z-back. This means "forward" is -Z.
    // Meshes always rotate around their local [0, 0, 0] when a rotation is applied to their Transform.
    // By centering our mesh around the origin, rotating the mesh preserves its center of mass.
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, attribute_position)
    // Set-up UV coordinates to point to the upper (V < 0.5), "dirt+grass" part of the texture.
    // Take a look at the custom image (assets/textures/array_texture.png)
    // so the UV coords will make more sense
    // Note: (0.0, 0.0) = Top-Left in UV mapping, (1.0, 1.0) = Bottom-Right in UV mapping
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, attribute_uv_0)
    // For meshes with flat shading, normals are orthogonal (pointing out) from the direction of
    // the surface.
    // Normals are required for correct lighting calculations.
    // Each array represents a normalized vector, which length should be equal to 1.0.
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, attribute_normal)
    // Create the triangles out of the 24 vertices we created.
    // To construct a square, we need 2 triangles, therefore 12 triangles in total.
    // To construct a triangle, we need the indices of its 3 defined vertices, adding them one
    // by one, in a counter-clockwise order (relative to the position of the viewer, the order
    // should appear counter-clockwise from the front of the triangle, in this case from outside the cube).
    // Read more about how to correctly build a mesh manually in the Bevy documentation of a Mesh,
    // further examples and the implementation of the built-in shapes.
    .with_inserted_indices(Indices::U32(indices))
}
