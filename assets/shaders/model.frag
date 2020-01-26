#version 330 core

struct PointLight {
    vec3 position;
    vec3 color;
    float strength;
};

uniform PointLight light;
uniform sampler2D TexFace;

in VS_OUTPUT {
    vec2 Uv;
    vec3 Normal;
    vec3 CameraPos;
    vec3 Position;
} IN;

out vec4 Color;

void main()
{
    //vec3 normal = normalize(IN.Normal);
    //vec3 lightDirection = normalize(light.position - IN.Position);
    //float diff = max(dot(normal, lightDirection), 0.0);
    //vec3 diffuse = diff * light.color * light.strength;
    vec3 color = texture(TexFace, IN.Uv).rgb; // * diffuse

    Color = vec4(color, 1.0);
}
