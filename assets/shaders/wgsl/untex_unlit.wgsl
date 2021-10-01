[[block]]
struct MatFactors {
    diffuse: vec4<f32>;
    emissive: vec4<f32>;
};

[[group(2), binding(0)]]
var<uniform> mat_factors: MatFactors;

struct VertexOutput {
    [[location(0)]] frag_pos: vec4<f32>;
    [[location(1)]] frag_norm: vec3<f32>;
    [[location(2)]] tex_coord: vec2<f32>;
    [[location(3)]] cam_pos: vec3<f32>;
};

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return mat_factors.diffuse;
}