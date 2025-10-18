#version 460

layout (binding = 0) uniform sampler2D leftImage;
layout (binding = 1) uniform sampler2D rightImage;

layout (binding = 2) uniform float separator;

layout (location = 0) in vec2 texPosition;

layout (location = 0) out vec4 outColor;

void main() {
    outColor = texture(leftImage, texPosition);
}