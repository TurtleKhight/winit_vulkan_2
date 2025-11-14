#version 450

// Vertex IN
layout(location = 0) in vec2 position;

// Fragment OUT
layout(location = 0) out vec2 f_clip;

void main() { 
    f_clip = position;
    gl_Position = vec4(position, 1.0, 1.0);
}
