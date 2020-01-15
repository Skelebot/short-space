#version 330 core

layout (location = 0) in vec3 in_position;
layout (location = 1) in vec4 in_color;
layout (location = 2) in vec2 in_tex_coord;

out vec4 f_color;
out vec2 tex_coord;

uniform mat4 mvp_matrix;

void main()
{
    gl_Position = mvp_matrix * vec4(in_position, 1.0);
    f_color = in_color;
    tex_coord = in_tex_coord;
}
