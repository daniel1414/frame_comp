use anyhow::Result;
use vulkanalia::prelude::v1_3::*;

pub fn create_framebuffer(
    device: &Device,
    render_pass: &vk::RenderPass,
    image_view: &vk::ImageView,
    extent: &vk::Extent2D,
) -> Result<vk::Framebuffer> {
    let attachments = &[*image_view];
    let create_info = vk::FramebufferCreateInfo::builder()
        .render_pass(*render_pass)
        .attachments(attachments)
        .width(extent.width)
        .height(extent.height)
        .layers(1)
        .build();

    let fb = unsafe { device.create_framebuffer(&create_info, None)? };
    Ok(fb)
}
