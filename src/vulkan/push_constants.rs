use crate::Color;

// The Push constant buffer's size must not exceed 128 bytes as it's one of the requirements of Vulkan.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PushConstantBuffer {
    pub color: Color,
    pub divider_pos: f32,
    pub divider_width: f32,
}
