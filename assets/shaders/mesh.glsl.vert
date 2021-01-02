#version 450

layout (location = 0) in vec3 a_Position;
layout (location = 1) in vec3 a_Normal;
layout (location = 2) in vec2 a_TexCoord;

layout (set = 0, binding = 0) uniform Globals {
    mat4 u_ViewProj;
    vec3 u_CamPos;
    uint u_NumLights;
};
layout (set = 1, binding = 0) uniform Mesh {
    mat4 u_Model;
};

layout (location = 0) out vec4 frag_pos;
layout (location = 1) out vec3 frag_norm;
layout (location = 2) out vec2 tex_coord;
layout (location = 3) out vec3 cam_pos;

void main() {
    
    gl_Position = u_ViewProj * u_Model * vec4(a_Position, 1.0);

    frag_pos = u_Model * vec4(a_Position, 1.0);
    // Protects aganist non-uniform scaling
    frag_norm = mat3(transpose(inverse(u_Model))) * a_Normal;

    tex_coord = a_TexCoord;
    cam_pos = u_CamPos;
}