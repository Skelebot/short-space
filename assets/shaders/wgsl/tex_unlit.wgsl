[[block]]
struct MatFactors {
    diffuse: vec4<f32>;
    emissive: vec4<f32>;
};

[[group(2), binding(0)]]
var<uniform> mat_factors: MatFactors;

[[group(2), binding(1)]]
var tex_sampler: sampler;

[[group(2), binding(2)]]
var tex_color: texture_2d<f32>;

struct VertexOutput {
    [[location(0)]] frag_pos: vec4<f32>;
    [[location(1)]] frag_norm: vec3<f32>;
    [[location(2)]] tex_coord: vec2<f32>;
    [[location(3)]] cam_pos: vec3<f32>;
};

[[stage(fragment)]]
fn main(
    in: VertexOutput
) -> [[location(0)]] vec4<f32> {

    let tex_color = textureSample(tex_color, tex_sampler, in.tex_coord);
    
    let final_color: vec4<f32> = mat_factors.diffuse * tex_color;
    
    return final_color;
}