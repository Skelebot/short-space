[[block]]
struct Globals {
    view_proj: mat4x4<f32>;
    cam_pos: vec3<f32>;
};

[[block]]
struct Mesh {
    model: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> global: Globals;

[[group(1), binding(0)]]
var<uniform> mesh: Mesh;

struct VertexOutput {
    [[location(0)]] frag_pos: vec4<f32>;
    [[location(1)]] frag_norm: vec3<f32>;
    [[location(2)]] tex_coord: vec2<f32>;
    [[location(3)]] cam_pos: vec3<f32>;
    // gl_Position
    [[builtin(position)]] vert_position: vec4<f32>;
};

[[stage(vertex)]]
fn main(
    [[location(0)]] in_position: vec3<f32>,
    [[location(1)]] in_normal: vec3<f32>,
    [[location(2)]] in_tex_coord: vec2<f32>
) -> VertexOutput {
    let vert_position = ((global.view_proj * mesh.model) * vec4<f32>(in_position, 1.0));

    let frag_pos = (mesh.model * vec4<f32>(in_position, 1.0));

    let frag_norm = in_normal;
    let tex_coord = in_tex_coord;

    let cam_pos = global.cam_pos;

    return VertexOutput(frag_pos, frag_norm, tex_coord, cam_pos, vert_position);
}
