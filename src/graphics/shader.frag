#version 450

layout (location = 0) in vec3 frag_pos;
layout (location = 1) in vec3 frag_norm;
layout (location = 2) in vec2 tex_coord;
layout (location = 3) in vec3 cam_pos;

layout(location = 0) out vec4 outColor;

layout(set = 1, binding = 1) uniform sampler s_Color;
layout(set = 1, binding = 2) uniform texture2D t_Color;

void main() {
    outColor = texture(sampler2D(t_Color, s_Color), tex_coord); // * in_diffuse (from mesh data)
    //outColor = vec4(1.0, 1.0, 1.0, 1.0);
}