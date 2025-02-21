use bevy::{
    ecs::{component::Component, system::Commands},
    pbr::StandardMaterial,
    render::mesh::Mesh,
};
use normal::NormalGenerator;

pub mod normal;

pub trait MapGenerator {
    fn generate_block(&self, x: i32, y: i32, z: i32) -> Mesh;
    fn generate_height_map(&self, x: i32, y: i32, z: i32) -> Vec<Vec<f32>>;
}

#[derive(Component, Debug)]
pub struct MapGeneratorInfo {
    name: String,
}

impl MapGeneratorInfo {
    pub fn region_generate(&self, region_x: i32, region_y: i32, region_z: i32) -> Mesh {
        NormalGenerator {}.generate_block(region_x, region_y, region_z)
    }

    pub fn height_map(&self, region_x: i32, region_y: i32, region_z: i32) -> Vec<Vec<f32>> {
        NormalGenerator{}.generate_height_map(region_x, region_y, region_z)
    }
}

pub fn setup(mut commands: Commands) {
    commands.spawn(MapGeneratorInfo {
        name: "all".to_string(),
    });
}
