#version 450

// Fragment IN
layout(location = 0) in vec2 f_clip;

// Uniforms
layout(set = 0, binding = 0) uniform Data {
    mat4 pv_inv_mtx;
    vec4 sky_colour;
    vec4 ground_colour;
} sky;

// Colour Attachments
layout(location = 0) out vec4 albedo;

void main() {
    albedo = vec4(1.0, 0.0, 0.0, 1.0);
    vec4 clip = vec4(f_clip, 0.0, 1.0);

    vec4 world_pos = sky.pv_inv_mtx * clip;
    vec3 view_dir = normalize(world_pos.xyz / world_pos.w);

    float t = clamp((view_dir.y+0.2)*10.0, 0.0, 1.0);
    vec3 colour = mix(sky.ground_colour.rgb, sky.sky_colour.rgb, t);

    albedo = vec4(colour, 1.0);
}