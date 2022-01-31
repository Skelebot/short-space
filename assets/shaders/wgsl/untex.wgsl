
struct Globals {
    view_proj: mat4x4<f32>;
    cam_pos: vec3<f32>;
};


struct MatFactors {
    diffuse: vec4<f32>;
    emissive: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> global: Globals;

[[group(2), binding(0)]]
var<uniform> mat_factors: MatFactors;


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

    // constants
    let sun_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let sun_color = vec3<f32>(1.0, 1.0, 1.0);
    let ambient_color = vec4<f32>(0.25, 0.25, 0.25, 1.0);

    let direction_diff = max(0.0, dot(normalize(in.frag_norm), sun_dir));
    
    let color_from_sun = direction_diff * sun_color;
    
    let final_color: vec4<f32> = (ambient_color + vec4<f32>(color_from_sun, 1.0)) * mat_factors.diffuse;
    
    return final_color;
}