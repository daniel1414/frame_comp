#version 460

layout (binding = 0) uniform sampler2D leftImage;
layout (binding = 1) uniform sampler2D rightImage;

layout (push_constant) uniform ComparatorPC {
    // Sort the types descending by size to avoid alignment issues
    vec4 divider_color;
    float divider;
    float divider_width;
} pc;

layout (location = 0) in vec2 texPosition;

layout (location = 0) out vec4 outColor;

void main() {

    if (texPosition.x < pc.divider - pc.divider_width / 2.0) {
        outColor = vec4(0.1, 0.0, 0.0, 1.0) + texture(leftImage, texPosition);
    } else if (texPosition.x > pc.divider + pc.divider_width / 2.0) {
        outColor = vec4(0.0, 0.1, 0.0, 1.0) + texture(rightImage, texPosition);
    } else {
        outColor = pc.divider_color;
    }
}