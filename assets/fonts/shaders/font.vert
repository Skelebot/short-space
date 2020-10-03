#version 330 core

layout (location = 0) in vec2 Position;
layout (location = 1) in vec2 Uv;

out VS_OUTPUT {
  vec2 Uv;
  vec2 Position;
} OUT;

void main() {
  gl_Position = vec4(Position, 0.0, 1.0);

  OUT.Uv = Uv;
  OUT.Position = Position;
}
