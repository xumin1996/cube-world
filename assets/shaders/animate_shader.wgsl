#import bevy_pbr::forward_io::{VertexOutput, FragmentOutput};
#import bevy_pbr::mesh_view_bindings::globals
#import bevy_render::view::View

/// Keep up-to-date with the rust definition!
struct AuraMaterial {
    unused: f32,
}

@group(0) @binding(0)   var<uniform> view: View;
@group(2) @binding(100) var<uniform> aura_mat: AuraMaterial;

// Colour picker tells us the values of the original..
// Darkish
// #CEAA4F
const GOLD = vec3f(0.807843, 0.666667, 0.309804);
const SPIKE_NUM: f32 = 9.0;
const SPIKE_LEN: f32 = 0.68;
const SPIKE_SPEED:f32 = 32.0;
const PI: f32 =  3.141592653589;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    uv = uv * 2.0 - 1.0;
    let x =(atan2(uv.x, uv.y) / PI + 1) * SPIKE_NUM; // Divide the x coords by PI so they line up perfectly.

    // 计算光针边缘
    let f_x = fract(x);
    var m = min(f_x, 1.0 - f_x);
    m = m  + 0.5*length(uv);
    
    // 计算当前像素值:
    var c = smoothstep(0.5,  0.0, m);
    var col = vec3f(c);

    // 全局时间计算指针位置
    let time = globals.time;
    let time_circle_index = floor(time * SPIKE_SPEED) % (SPIKE_NUM * 2.0);
    let focus_length = min(abs(time_circle_index - x), abs(time_circle_index + 2*SPIKE_NUM - x));
    let is_focused_spike = step(0.5, focus_length);
    col *= mix(GOLD / 0.15, GOLD * 0.54, is_focused_spike);

    // 不显示中间
    let feet_mask = length(uv) - 0.25;
    col *= smoothstep(0.0, 0.09, feet_mask);

    // 输出
    var out = vec4f(col, 1.0);
    return out;
}