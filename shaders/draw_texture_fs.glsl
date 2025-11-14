#version 450

// Fragment IN
layout(location = 0) in vec2 f_clip;

// Uniforms
layout(set = 0, binding = 0) uniform sampler s;
layout(set = 0, binding = 1) uniform texture2D t;

// Colour Attachments
layout(location = 0) out vec4 final_colour;

void main() {
    vec2 uv = f_clip*0.5+0.5;
    vec4 albedo = texture(sampler2D(t, s), uv);
    final_colour = albedo;
}