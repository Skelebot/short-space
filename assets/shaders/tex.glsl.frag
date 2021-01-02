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

#define sunColor vec3(1.0, 1.0, 1.0)
#define sunDir vec3(-1.0, -1.0, -1.0)

void main() {
    vec3 tex_color = texture(sampler2D(t_Color, s_Color), tex_coord).xyz; // * in_diffuse (from mesh data)

    float ambientStrength = 0.25;
    vec3 l_ambient = ambientStrength * sunColor;

    vec3 lightDir = normalize(-sunDir);
    float diff = max(dot(frag_norm, lightDir), 0.0);
    vec3 l_diffuse = diff * sunColor;

    vec3 result = (l_ambient + l_diffuse) * tex_color;

    outColor = vec4(result, 1.0);
}