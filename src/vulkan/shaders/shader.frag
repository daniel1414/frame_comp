#version 460

layout (binding = 0) uniform sampler2D leftImage;
layout (binding = 1) uniform sampler2D rightImage;

layout (location = 0) out vec3 outColor;

void main() {
    outColor = vec3(0.4, 0.5, 0.1);
}