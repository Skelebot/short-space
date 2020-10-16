#version 450

layout (location = 0) in vec3 frag_pos;
layout (location = 1) in vec3 frag_norm;
layout (location = 3) in vec3 cam_pos;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(1.0, 1.0, 1.0, 1.0);
}