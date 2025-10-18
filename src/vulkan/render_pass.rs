use anyhow::Result;
use vulkanalia::prelude::v1_3::*;

pub fn create_render_pass(
    device: &Device,
    format: vk::Format,
    final_layout: Option<vk::ImageLayout>,
) -> Result<vk::RenderPass> {
    let final_layout = match final_layout {
        Some(layout) => layout,
        None => vk::ImageLayout::PRESENT_SRC_KHR,
    };

    let color_attachment = vk::AttachmentDescription::builder()
        // Format of the color attachment should be same as the swapchain images.
        .format(format)
        // We don't multipsample in the comparator, as we're just copying pixels one by one.
        .samples(vk::SampleCountFlags::_1)
        // Defines what happens to the attachment at the start of rendering
        .load_op(vk::AttachmentLoadOp::CLEAR)
        // What happens to the attachment after rendering
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        // Expected layout of the attachment before rendering.
        .initial_layout(vk::ImageLayout::UNDEFINED)
        // Defines what the final layout of the attachment should be after rendering.
        .final_layout(final_layout)
        .build();

    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        // While this subpass is running, the attachment will be in this layout.
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(std::slice::from_ref(&color_attachment_ref))
        .build();

    // This dependency makes sure that the swapchain image is ready to be written to
    // in the first subpass. Ensures pipeline and memory synchronization.
    let dependency = vk::SubpassDependency::builder()
        // The source of the dependency is not part of the render pass. It refers to operations
        // outside the render pass, such as image layout transitions after aquiring a swapchain image
        // and any preceding rendering or compute operations that could affect the attachments.
        .src_subpass(vk::SUBPASS_EXTERNAL)
        // The destination subpass is the one and only we have, which has been created above.
        .dst_subpass(0)
        // Specifies the pipeline stage(s) in the source scope that need to be synchronized.
        // COLOR_ATTACHMENT_OUTPUT represents the stage where color attachment writes occur.
        // In this case, Vulkan ensures that color attachment output from operations
        // outside the render pass is finished before continuing (e.g. presenting to the user).
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        // Specifies the memory access type(s) in the source scope that need synchronization.
        // In this case there are no specific memory accesses that need synchronization in this dependency.
        .src_access_mask(vk::AccessFlags::empty())
        // Specifies the pipeline stage(s) in the destination scope that depend on the source.
        // Again, this is COLOR_ATTACHMENT_OUTPUT, meaning the rendering commands in the first
        // subpass that write to the color attachment depend on the completion of prior operations.
        // This ensures that the destination subpass starts writing to the color attachment
        // only after it's safe to do so.
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        // Specifies the memory access type(s) required in the destination scope.
        // COLOR_ATTACHMENT_WRITE indicates that the subpass will write to the color attachment.
        // This ensures proper synchronization of memory for writing, so the render pass
        // doesn't overwrite data that's still being processed from prior operations.
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

    let rp_info = vk::RenderPassCreateInfo::builder()
        .attachments(std::slice::from_ref(&color_attachment))
        .subpasses(std::slice::from_ref(&subpass))
        .dependencies(std::slice::from_ref(&dependency));

    Ok(unsafe { device.create_render_pass(&rp_info, None) }?)
}
