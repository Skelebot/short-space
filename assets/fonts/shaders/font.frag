#version 330 core

uniform sampler2D TexFace;

in VS_OUTPUT {
  vec2 Uv;
  vec2 Position;
} IN;

out vec4 Color;

void main() {
  vec3 color = texture(TexFace, IN.Uv).rgb;
  Color = vec4(color, 1.0);
}
