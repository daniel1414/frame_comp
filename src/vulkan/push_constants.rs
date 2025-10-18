#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PushConstantBuffer {
    pub divider_pos: f32,
    pub divider_width: f32,
}
