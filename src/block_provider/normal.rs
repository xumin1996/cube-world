use std::time::Instant;

use crate::util::Triangle;

use super::MapGenerator;
use bevy::{
    ecs::{component::Component, system::Commands},
    math::{primitives::Cuboid, Vec3},
    pbr::StandardMaterial,
    render::mesh::{Mesh, MeshBuilder, Meshable},
    transform::components::Transform,
};
use simdnoise::NoiseBuilder;

pub struct NormalGenerator;

impl MapGenerator for NormalGenerator {
    fn generate_block(&self, region_x: i32, region_y: i32, region_z: i32) -> Mesh {
        region_by_block(region_x, region_z)
    }
    fn generate_height_map(&self, region_x: i32, region_y: i32, region_z: i32) -> Vec<Vec<f32>> {
        height_map_by_region(region_x, 0, region_z)
    }
}

fn height_map_by_region(region_x: i32, region_y: i32, region_z: i32) -> Vec<Vec<f32>> {
    let plain_size = 16i32;
    // [x1,x1,x1,...,x2,x2,x2,...,x3,x3,x3,....xy, xy,xy,...]
    let (heights, min, max) = NoiseBuilder::fbm_2d_offset(
        (region_z * plain_size) as f32,
        (plain_size) as usize,
        (region_x * plain_size) as f32,
        (plain_size) as usize,
    )
    .with_seed(1)
    .with_freq(0.01)
    .generate();
    let heights: Vec<f32> = heights.iter().map(|item| (item * 200f32).floor()).collect();
    let plain_height: Vec<Vec<f32>> = heights
        .chunks((plain_size) as usize)
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
    .with_freq(0.01)
    .generate();
    let heights: Vec<f32> = heights.iter().map(|item| (item * 200f32).floor()).collect();
    let plain_height: Vec<Vec<f32>> = heights
        .chunks((plain_size + 1) as usize)
        .map(|chunk| {
            let mut rv = chunk.to_vec();
            rv
        })
        .collect();
    return plain_height;
}

fn create_cube_mesh(height_mesh: &Vec<Vec<f32>>) -> Mesh {
    let plain_size = 16usize;
    let mut cube_transform: Vec<Transform> = Vec::<Transform>::new();
    let mut cube_triangles: Vec<Triangle> = Vec::<Triangle>::new();

    for (x_index, z_list) in height_mesh.iter().take(plain_size).enumerate() {
        for (z_index, y_height) in z_list.iter().take(plain_size).enumerate() {
            // 顶点
            let cube_size = 1f32;
            let x = cube_size * x_index as f32;
            let y = *y_height;
            let z = cube_size * z_index as f32;

            let mut cube_mesh: Triangle =
                Triangle::from_mesh(&Cuboid::new(1.0, 1.0, 1.0).mesh().build());
            cube_mesh.uv.iter_mut().for_each(|item: &mut Vec3| {
                item.x = item.x / plain_size as f32 + x_index as f32 / plain_size as f32;
                item.y = item.y / plain_size as f32 + z_index as f32 / plain_size as f32;
            });
            cube_triangles.push(cube_mesh * Transform::from_xyz(x, y, z));
        }
    }

    let mut r: Triangle = Triangle::new(Vec::new(), Vec::new(), Vec::new());
    for tri in cube_triangles {
        r = r + tri;
    }

    r.build()
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
