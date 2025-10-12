#version 460

layout (binding = 0) uniform sampler2D leftImage;
layout (binding = 1) uniform sampler2D rightImage;

layout (location = 0) in vec4 pos;
layout (location = 1) in vec3 inColor;

layout (location = 0) out vec3 outColor;

void main() {
    outColor = inColor;
}