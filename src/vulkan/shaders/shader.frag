#version 460

layout (binding = 0) uniform sampler2D leftImage;
layout (binding = 1) uniform sampler2D rightImage;

layout (push_constant) uniform ComparatorPC {
    float divider;
} pc;

layout (location = 0) in vec2 texPosition;

layout (location = 0) out vec4 outColor;

void main() {
    if (texPosition.x < pc.divider) {
        outColor = vec4(0.1, 0.0, 0.0, 1.0) + texture(leftImage, texPosition);
    } else {
        outColor = vec4(0.0, 0.1, 0.0, 1.0) + texture(rightImage, texPosition);
    }
}