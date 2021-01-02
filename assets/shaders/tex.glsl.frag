#version 450

layout (location = 0) in vec3 frag_pos;
layout (location = 1) in vec3 frag_norm;
layout (location = 2) in vec2 tex_coord;
layout (location = 3) in vec3 cam_pos;

layout(set = 2, binding = 0) uniform MatFactors {
    vec4 u_Diffuse;
    vec4 u_Emissive;
};

layout(set = 2, binding = 1) uniform sampler s_Color;
layout(set = 2, binding = 2) uniform texture2D t_Color;

layout(location = 0) out vec4 outColor;

#define sunDir normalize(vec3(1.0, 1.0, 1.0))
#define sunColor vec3(1.0, 1.0, 1.0)

void main() {
    vec3 norm = normalize(frag_norm);
    vec3 ambient = vec3(0.25, 0.25, 0.25);

    float diff = max(0.0, dot(norm, sunDir));
    vec3 diffuse = diff * sunColor;

    vec3 color = ambient + diffuse;
    vec4 tex_color = texture(sampler2D(t_Color, s_Color), tex_coord);
    outColor = vec4(color, 1.0) * u_Diffuse * tex_color;
}