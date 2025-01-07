#import bevy_pbr::mesh_view_bindings::globals
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

const GRID_RATIO:f32 = 40.;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = globals.time;
    var uv = in.uv - 0.5;
    var col = vec3(0.0);

    uv *= 10.;
    let grid = grid(uv);
    let pal = palette(t / 2. );
    col = mix(col, pal, grid);
   
    return vec4<f32>(col, 1.0);
}

// 变幻彩色
fn palette(time : f32) -> vec3<f32> {
    let a = vec3<f32>(0.5, 0.5, 0.5);
    let b = vec3<f32>(0.5, 0.5, 0.5);
    let c = vec3<f32>(1.0, 1.0, 1.0);
    let d = vec3<f32>(0.263, 0.416, 0.557);

    return a + b * cos(6.28318 * (c * time + d));
}

// 显示网格
fn grid(uv: vec2<f32>)-> f32 {
    let i = step(fract(uv), vec2(1.0/GRID_RATIO));
    return max(i.x, i.y);
}

fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    let K: vec4<f32> = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    var p: vec3<f32> = abs(fract(vec3<f32>(c.x) + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), c.y);
}
