#version 450

layout (location = 0) in vec3 frag_pos;
layout (location = 1) in vec3 frag_norm;
layout (location = 2) in vec2 tex_coord;
layout (location = 3) in vec3 cam_pos;

layout(set = 2, binding = 0) uniform MatFactors {
    vec4 diffuse;
    vec4 emissive;
};

layout(set = 2, binding = 1) uniform sampler s_Color;
layout(set = 2, binding = 2) uniform texture2D t_Color;

layout(location = 0) out vec4 outColor;

void main() {
    vec4 tex_color = texture(sampler2D(t_Color, s_Color), tex_coord); // * in_diffuse (from mesh data)

    outColor = tex_color * emissive * diffuse;
}