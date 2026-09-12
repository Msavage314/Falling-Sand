struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // fullscreen triangle, no buffer needed
    var out: VertexOutput;
    let x= f32((vertex_index << 1u) & 2u);
    let y= f32(vertex_index & 2u);
    out.uv = vec2<f32>(x,y);
    out.clip_position = vec4<f32>(x* 2.0 - 1.0, 1.0-y *2.0, 0.0,1.0);
    return out;
}

@group(0) @binding(0) var t_source: texture_2d<f32>;
@group(0) @binding(1) var s_source: sampler;

struct BlurParams {
    direction_texel: vec2<f32>,
    _padding: vec2<f32>,
};
@group(0) @binding(2) var<uniform> params: BlurParams;

const weights = array<f32, 5>(0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_source, s_source, in.uv) * weights[0];
    for (var i = 1; i < 5; i= i+1){
        let offset = params.direction_texel * f32(i);
        color += textureSample(t_source, s_source, in.uv + offset) * weights[i];
        color += textureSample(t_source, s_source, in.uv - offset) * weights[i];
    }
    return color;
}