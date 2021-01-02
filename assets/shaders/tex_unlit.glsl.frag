#version 450
#extension GL_EXT_debug_printf : enable

const int MAX_LIGHTS = 5;

layout (location = 0) in vec4 frag_pos;
layout (location = 1) in vec3 frag_norm;
layout (location = 2) in vec2 tex_coord;
layout (location = 3) in vec3 cam_pos;

struct Light {
    mat4 proj;
    vec4 pos;
    vec4 color;
};

layout (set = 0, binding = 0) uniform Globals {
    mat4 u_ViewProj;
    vec3 u_CamPos;
    uint u_NumLights;
};

layout (set = 0, binding = 1) uniform Lights {
    Light u_Lights[MAX_LIGHTS];
};

layout(set = 0, binding = 2) uniform texture2DArray t_Shadow;
layout(set = 0, binding = 3) uniform samplerShadow s_Shadow;

layout(set = 2, binding = 0) uniform MatFactors {
    vec4 u_Diffuse;
    vec4 u_Emissive;
};

layout(set = 2, binding = 1) uniform sampler s_Color;
layout(set = 2, binding = 2) uniform texture2D t_Color;

layout(location = 0) out vec4 outColor;

float fetch_shadow(int light_id, vec4 homogeneous_coords) {
    if (homogeneous_coords.w <= 0.0) {
        return 1.0;
    }
    // compensate for the Y-flip difference between the NDC and texture coordinates
    const vec2 flip_correction = vec2(0.5, -0.5);
    // compute texture coordinates for shadow lookup
    vec4 light_local = vec4(
        homogeneous_coords.xy * flip_correction/homogeneous_coords.w + 0.5,
        light_id,
        homogeneous_coords.z / homogeneous_coords.w
    );
    // do the lookup, using HW PCF and comparison
    return texture(sampler2DArrayShadow(t_Shadow, s_Shadow), light_local);
}

void main() {
    vec3 normal = normalize(frag_norm);
    vec3 ambient = vec3(0.05, 0.05, 0.05);
    // accumulate color
    vec3 color = ambient;
    for (int i=0; i<int(u_NumLights) && i<MAX_LIGHTS; ++i) {
        Light light = u_Lights[i];
        // project into the light space
        float shadow = fetch_shadow(i, light.proj * frag_pos);
        // compute Lambertian diffuse term
        vec3 light_dir = normalize(light.pos.xyz - frag_pos.xyz);
        float diffuse = max(0.0, dot(normal, light_dir));
        // add light contribution
        color += shadow * diffuse * light.color.xyz;

        if (gl_FragCoord.xy == vec2(0.5, 0.5)) {
            debugPrintfEXT("\nlight_num: %i shadow: %f, pos: %v4f, color: %v4f", i, shadow, light.pos, light.color);
        }
    }
    // multiply the light by material color
    outColor = vec4(color, 1.0) * u_Diffuse;
}