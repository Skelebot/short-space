#version 450

layout (location = 0) in vec3 Position;
layout (location = 1) in vec3 Normal;
//layout (location = 2) in vec4 Tangent;
layout (location = 2) in vec2 TexCoord;

layout (set = 0, binding = 0) uniform Globals {
    mat4 u_ViewProj;
    vec3 u_CamPos;
};
layout (set = 1, binding = 0) uniform Mesh {
    mat4 u_Model;
};

layout (location = 0) out vec3 frag_pos;
layout (location = 1) out vec3 frag_norm;
layout (location = 2) out vec2 tex_coord;
layout (location = 3) out vec3 cam_pos;

void main() {
    
    gl_Position = u_ViewProj * u_Model * vec4(Position, 1.0);

    // Protects aganist non-uniform scaling
    frag_norm = mat3(transpose(inverse(u_Model))) * Normal;
    cam_pos = u_CamPos;
    frag_pos = Position;
    tex_coord = TexCoord;
}