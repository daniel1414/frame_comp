#version 460

layout (binding = 0) uniform sampler2D leftImage;
layout (binding = 1) uniform sampler2D rightImage;

layout (location = 0) out vec4 outColor;

void main() {
    outColor = vec4(0.9, 0.5, 0.1, 0.5);
}