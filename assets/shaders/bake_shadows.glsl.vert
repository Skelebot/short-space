#version 450

layout (location = 0) in vec3 Position;

layout (set = 0, binding = 0) uniform Globals {
    mat4 u_ViewProj;
};

layout (set = 1, binding = 0) uniform Mesh {
    mat4 u_Model;
};

void main() {
    gl_Position = u_ViewProj * u_Model * vec4(Position, 1.0);
}