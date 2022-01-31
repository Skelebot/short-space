
struct Globals {
    view_proj: mat4x4<f32>;
    cam_pos: vec3<f32>;
};


struct DebugLinesUniforms {
    thickness: f32;
};

struct VertexOutput {
    [[location(0)]] color: vec4<f32>;
    [[builtin(position)]] member: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> globals: Globals;
[[group(0), binding(1)]]
var<uniform> debug_lines_uniforms: DebugLinesUniforms;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] pos_a: vec3<f32>,
    [[location(1)]] color_a: vec4<f32>,
    [[location(2)]] pos_b: vec3<f32>,
    [[location(3)]] color_b: vec4<f32>,
    [[builtin(vertex_index)]] vertex_id: u32)
-> VertexOutput {
    
    let proj_a = globals.view_proj * vec4<f32>(pos_a, 1.0);
    let proj_b = globals.view_proj * vec4<f32>(pos_b, 1.0);
    let factor = f32((vertex_id >> u32(1)));
    let proj_current = mix(proj_a, proj_b, vec4<f32>(factor));
    
    var position: vec4<f32>;
    
    if (proj_current.w < f32(0)) {
        // vertex is behind the camera clip plane
        let proj_next: vec4<f32> = mix(proj_b, proj_a, vec4<f32>(factor));
        let clip_space_dir: vec3<f32> = normalize(proj_current.xyw - proj_next.xyw);
        let coef = -proj_current.w / clip_space_dir.z;
        let intersect_pos: vec3<f32> = proj_current.xyw + (clip_space_dir * coef);
        position = vec4<f32>(intersect_pos.x, intersect_pos.y, 0.0, intersect_pos.z);
    } else {
        let screen_a: vec2<f32> = proj_a.xy / proj_a.w;
        let screen_b: vec2<f32> = proj_b.xy / proj_b.w;
        let dir = normalize(screen_b - screen_a);
        
        var normal: vec2<f32>;
        if (vertex_id % u32(2) == u32(0)) {
            normal = vec2<f32>(-dir.y, dir.x);
        } else {
            normal = vec2<f32>(dir.y, -dir.x);
        }
        
        normal = normal * (proj_current.w * vec2<f32>(debug_lines_uniforms.thickness));

        position = proj_current + vec4<f32>(normal, 0.0, 0.0);
    }

    let color: vec4<f32> = mix(color_a, color_b, vec4<f32>(factor));

    return VertexOutput(color, position);
}

struct FragmentOutput {
    [[location(0)]] outColor: vec4<f32>;
};

[[stage(fragment)]]
fn fs_main([[location(0)]] color: vec4<f32>) -> FragmentOutput {
    return FragmentOutput(color);
}