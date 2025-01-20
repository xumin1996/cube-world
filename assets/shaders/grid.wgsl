#import bevy_pbr::mesh_view_bindings::globals
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

const GRID_RATIO:f32 = 40.;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = globals.time;
    var uv = in.uv - 0.5;
    var col = vec3(0.129, 0.651, 0.208);

    uv *= 10.;
    let grid = grid(uv);
    let pal = vec3<f32>(0.5, 0.5, 0.5);
    col = mix(col, pal, grid);
   
    return vec4<f32>(col, 1.0);
}

// 显示网格
fn grid(uv: vec2<f32>)-> f32 {
    let i = step(fract(uv), vec2(1.0/GRID_RATIO));
    return max(i.x, i.y);
}
