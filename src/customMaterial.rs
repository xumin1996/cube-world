use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CustomMaterial {}

impl Material for CustomMaterial {
    // fn vertex_shader() -> ShaderRef {
    //     "shaders/animate_shader.wgsl".into()
    // }
    fn fragment_shader() -> ShaderRef {
        "shaders/grid.wgsl".into()
    }
}
