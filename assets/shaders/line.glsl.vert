#version 450

layout (location = 0) in vec3 pos_a;
layout (location = 1) in vec4 color_a;
layout (location = 2) in vec3 pos_b;
layout (location = 3) in vec4 color_b;

layout (set = 0, binding = 0) uniform Globals {
    mat4 view_proj;
    vec3 cam_pos;
};

layout (set = 0, binding = 1) uniform DebugLinesUniforms {
    float thickness;
};

layout(location = 0) out VertexData {
    vec4 color;
} vertex;

void main() {

    float factor = float(gl_VertexIndex >> 1);
    // Returns color_a or color_b depending on which end of the line we are on
    vertex.color = mix(color_a, color_b, factor);
    
    vec4 proj_a = view_proj * vec4(pos_a, 1.0);
    vec4 proj_b = view_proj * vec4(pos_b, 1.0);
    vec4 proj_current = mix(proj_a, proj_b, factor);
    
    if (proj_current.w < 0) {
        // vertex behind camera clip plane
        vec4 proj_next = mix(proj_b, proj_a, factor);
        vec3 clip_space_dir = normalize(proj_current.xyw - proj_next.xyw);
        float coef = -proj_current.w / clip_space_dir.z;
        vec3 intersect_pos = proj_current.xyw + (clip_space_dir * coef);
        gl_Position = vec4(intersect_pos.x, intersect_pos.y, 0, intersect_pos.z);
    } else {
        vec2 screen_a = proj_a.xy / proj_a.w;
        vec2 screen_b = proj_b.xy / proj_b.w;
        vec2 dir = normalize(screen_b - screen_a);
        
        vec2 normal;
        if (mod(gl_VertexIndex, 2) == 0) {
            normal = vec2(-dir.y, dir.x);
        } else {
            normal = vec2(dir.y, -dir.x);
        }
        
        normal *= proj_current.w * vec2(thickness);
        gl_Position = proj_current + vec4(normal, 0.0, 0.0);
    }
}