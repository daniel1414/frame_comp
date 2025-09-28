use anyhow::Error;
use vulkanalia::prelude::v1_3::*;

pub mod vulkan;

#[derive(Clone, Debug)]
pub struct FrameComparatorCreateInfo<'a, 'b> {
    // The vulkan instance
    instance: &'a Instance,

    // For interfacing with the hardware,
    device: &'b Device,

    // For knowing where to render the output to.
    render_area: vk::Rect2D,

    // Render pass, just duh.
    swapchain_format: vk::Format,
    msaa_samples: vk::SampleCountFlags,
    depth_format: vk::Format,
}

impl<'a, 'b> FrameComparatorCreateInfo<'a, 'b> {
    pub fn new(
        instance: &'a Instance,
        device: &'b Device,
        render_area: vk::Rect2D,
        swapchain_format: vk::Format,
        depth_format: vk::Format,
        msaa_samples: vk::SampleCountFlags,
    ) -> Self {
        Self {
            instance,
            device,
            render_area,
            swapchain_format,
            msaa_samples,
            depth_format,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FrameComparator<'a, 'b> {
    create_info: FrameComparatorCreateInfo<'a, 'b>,
    render_pass: vk::RenderPass,
}

impl<'a, 'b> FrameComparator<'a, 'b> {
    pub fn new(info: FrameComparatorCreateInfo<'a, 'b>) -> Result<Self, Error> {
        let render_pass = vulkan::render_pass::create_render_pass(&info)?;

        Ok(Self {
            create_info: info,
            render_pass: render_pass,
        })
    }

    pub fn render(&self, command_buffer: &vk::CommandBuffer, percentage: f32) {
        let vbar_width = 4u32;
        let render_area = &self.create_info.render_area;

        let left_extent = (render_area.extent.width as f32 * percentage) as u32;
        let left_render_area = vk::Rect2D::builder()
            .offset(render_area.offset)
            .extent(vk::Extent2D {
                width: left_extent - vbar_width / 2,
                height: render_area.extent.height,
            })
            .build();

        let right_render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D {
                x: (left_extent + vbar_width / 2) as i32,
                y: 0,
            })
            .extent(vk::Extent2D {
                width: render_area.extent.width - left_extent - vbar_width / 2,
                height: render_area.extent.height,
            })
            .build();
    }
}
