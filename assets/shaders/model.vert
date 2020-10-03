#version 330 core

layout (location = 0) in vec3 Position;
layout (location = 1) in vec3 Normal;
layout (location = 2) in vec2 Uv;

uniform vec3 CameraPos;
uniform mat4 View;
uniform mat4 Projection;
uniform mat4 Model;

out VS_OUTPUT {
    vec2 Uv;
    vec3 Normal;
    vec3 CameraPos;
    vec3 Position;
} OUT;

void main()
{
    gl_Position = Projection * View * Model * vec4(Position, 1.0);

    OUT.Uv = Uv;
    //protect aganist non-uniform scaling
    OUT.Normal = mat3(transpose(inverse(Model))) * Normal;
    OUT.CameraPos = CameraPos;
    OUT.Position = Position;
}
