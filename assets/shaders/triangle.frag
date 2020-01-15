#version 330 core

in vec4 f_color;
in vec2 tex_coord;

out vec4 frag_color;

uniform sampler2D in_texture;

void main() {
  frag_color = mix(texture(in_texture, tex_coord), f_color, 0.5);
}
