
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct Camera {
    proj: mat4x4<f32>,
};
@group(0) @binding(0) 
var<uniform> camera: Camera;

// struct Sprite {
//     model: mat4x4<f32>,
//     color: vec4<f32>,
//     tex_scale: vec2<f32>,
//     tex_pos:  vec2<f32>
// }
// @group(1) @binding(0)
// var<uniform> sprite: Sprite;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    out.clip_position = camera.proj * vec4<f32>(in.position * vec2f(48.0, 48.0) + vec2f(400.0, 300.0) , 0.0, 1.0);
    out.tex_coords = vec2<f32>(in.tex_coords.x, in.tex_coords.y);
    out.color = vec4f(1.0,1.0,1.0,1.0);
    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords ) *in.color ;
}