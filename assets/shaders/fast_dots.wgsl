#import bevy_pbr::mesh_view_bindings globals
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv: vec2<f32> = in.uv;

    var m = 0.;
    let t: f32 = globals.time / 100;
    for (var i = 0; i < 30; i += 1) {
        let n:vec2<f32> = vec2(rand(t + f32(i )* 2.0), rand(t + 0.5 + f32(i) * 2.0));
        let d = length(uv - n);
        m += smoothstep(0.002, 0.001, d);
    }

    var col = vec3(m);
    return vec4(col, 1.0);
}

// fft构造的连续伪随机树
fn rand(x: f32) -> f32 {
    // fft 随机数
    var y = 0.1042*sin(1*x)+0.7563*cos(1*x)+0.4530*sin(2*x)+0.6678*cos(2*x)+0.8024*sin(4*x)+0.1780*cos(4*x)+0.2779*sin(8*x)+0.4869*cos(8*x)+0.2147*sin(16*x)+0.5170*cos(16*x)+0.0115*sin(32*x)+0.6059*cos(32*x)+0.9722*sin(64*x)+0.1842*cos(64*x)+0.9056*sin(128*x)+0.6755*cos(128*x)+0.1378*sin(256*x)+0.6769*cos(256*x)+0.2455*sin(512*x)+0.8363*cos(512*x);
    y = (y / 10) / 2 + 0.5;
    return min(max(0.0, y), 1.0);
}