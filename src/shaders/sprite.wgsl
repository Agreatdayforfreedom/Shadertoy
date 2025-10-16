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

struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
}
@group(2) @binding(0)
var<uniform> u: Uniforms;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = camera.proj * vec4<f32>(in.position * (u.resolution) + vec2f(400.0, 300.0), 0.0, 1.0);
    out.tex_coords = vec2<f32>(in.tex_coords.x, in.tex_coords.y);
    out.color = vec4f(1.0, 1.0, 1.0, 1.0);
    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

fn sdSphere(p: vec3<f32>, r: f32) -> f32 { return length(p) - r; }
fn sceneDist(p: vec3<f32>) -> f32 {
    // two spheres for interest
    let s1 = sdSphere(p - vec3<f32>(0.0, 0.0, 3.5 + 0.5 * sin(u.time)), 1.0);
    let s2 = sdSphere(p - vec3<f32>(1.5, 0.0, 4.0), 0.6);
    return min(s1, s2);
}
fn softShadow(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var res = 1.0;
    var t = 0.02;
    for (var i: i32 = 0; i < 40; i = i + 1) {
        let p = ro + rd * t;
        let h = sceneDist(p);
        if h < 0.001 { return 0.0; }
        res = min(res, 8.0 * h / t);
        t = t + clamp(h, 0.02, 0.2);
        if t > 20.0 { break; }
    }
    return clamp(res, 0.0, 1.0);
}

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898,78.233))) * 43758.5453);
}
@fragment
fn fs_main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {
    var uv = (v_uv - 0.5) * 2.0;
    uv.x *= u.resolution.x / u.resolution.y;

  
    let t = u.time * 0.05;
    let a = -atan2(uv.y, uv.x);
    let r = length(uv) * 2.0;
    let waves = sin(2.5*r - t*2.5);

    let col = 0.5 + 0.5 * vec3<f32>(
        sin(a*1.0 + (sin(t) * cos(t))),
        sin(a*2.0 + (t * sin(t))) + 2.0 * sin(t) * cos(a),
        sin(a*3.0 + (sin(t) * cos(t))) + 4.0 * cos(t) * sin(a)
    );

    let fade = 1.0/(1.0 + 4.0*r*r);
    //return vec4<f32>(col * fade * (waves + 1.0), 1.0);
  return vec4<f32>(col * fade * (waves + 5.0), 1.0) * 3.0;
}

