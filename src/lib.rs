//! A Vulkan utility for comparing two images with a draggable divider.
//!
//! This crate provides a `FrameComparator` struct that encapsulates the necessary
//! Vulkan resources to render a side-by-side comparison of two images into a
//! target image view.

use anyhow::Result;
use derive_builder::Builder;
use std::rc::Rc;
use vulkanalia::prelude::v1_3::*;

use crate::vulkan::{
    descriptors::{create_descriptor_set, create_descriptor_set_layout, update_descriptor_sets},
    pipeline::create_pipeline,
    push_constants::PushConstantBuffer,
    render_pass::create_render_pass,
    sampler::create_image_sampler,
};

pub(crate) mod vulkan;

/// A simple RGBA color struct.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color(pub f32, pub f32, pub f32, pub f32);

/// Configuration for a single frame comparison operation.
#[derive(Builder, Clone, Copy, Debug)]
#[builder(setter(into))]
pub struct FrameCompareInfo {
    /// The command buffer to record drawing commands into.
    #[builder(default)]
    pub command_buffer: vk::CommandBuffer,
    /// The horizontal position of the divider, in the range `[0.0, 1.0]`.
    #[builder(default = "0.5_f32")]
    pub divider_position: f32,
    /// The width of the divider line in pixels.
    #[builder(default = "4_u8")]
    pub divider_width: u8,
    /// The color of the divider line.
    #[builder(default)]
    pub divider_color: Color,
}

impl FrameCompareInfo {
    pub fn builder() -> FrameCompareInfoBuilder {
        FrameCompareInfoBuilder::default()
    }
}

/// A reusable Vulkan utility for rendering a side-by-side image comparison.
#[derive(Debug)]
pub struct FrameComparator {
    render_pass: vk::RenderPass,

    device: Rc<Device>,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    output_extent: vk::Extent2D,
    sampler: vk::Sampler,

    /// Caches framebuffers to avoid recreating them on every `compare` call.
    /// The `RefCell` allows for interior mutability.
    framebuffer: vk::Framebuffer,
}

impl Drop for FrameComparator {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_sampler(self.sampler, None);
            self.device.destroy_framebuffer(self.framebuffer, None);
            self.device.destroy_pipeline(self.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            // Descriptor sets are allocated from the pool and don't need to be freed individually.
            self.device.destroy_render_pass(self.render_pass, None);
        }
    }
}

impl FrameComparator {
    /// Returns the amount of image samplers that will be allocated by the frame comparator per compare() invocation.
    /// This needs to be taken into account when creating the descriptor pool.
    pub fn image_sampler_count() -> u32 {
        2
    }

    /// Creates a new `FrameComparator`.
    pub fn new(
        device: Rc<Device>,
        descriptor_pool: vk::DescriptorPool,
        format: vk::Format,
        extent: vk::Extent2D,
        final_layout: Option<vk::ImageLayout>,
        in_image_views: [vk::ImageView; 2],
        out_image_view: vk::ImageView,
    ) -> Result<Self> {
        let render_pass = create_render_pass(&device, format, final_layout)?;
        let descriptor_set_layout = create_descriptor_set_layout(&device)?;

        let (pipeline_layout, pipeline) =
            create_pipeline(&device, &extent, &render_pass, &[descriptor_set_layout])?;

        let sampler = create_image_sampler(&device)?;

        // Create framebuffer
        let attachments = &[out_image_view];
        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(attachments)
            .width(extent.width)
            .height(extent.height)
            .layers(1);

        // This is inside an unsafe function, and the caller guarantees the
        // validity of the image view.
        let framebuffer = unsafe { device.create_framebuffer(&framebuffer_info, None)? };

        // Handle descriptors
        let descriptor_set =
            create_descriptor_set(&device, &descriptor_pool, &descriptor_set_layout)?;

        update_descriptor_sets(&device, &descriptor_set, &sampler, &in_image_views);

        Ok(Self {
            render_pass,
            device,
            descriptor_set_layout,
            descriptor_set,
            pipeline_layout,
            pipeline,
            output_extent: extent,
            sampler,
            framebuffer,
        })
    }

    /// Records the drawing commands for comparing two images into the provided command buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the `descriptor_pool` provided during `FrameComparator`
    /// creation has enough capacity to allocate a new descriptor set for each call to `compare`.
    /// The allocated descriptor set is valid only for the lifetime of the provided command buffer.
    pub unsafe fn compare(&self, info: &FrameCompareInfo) -> Result<()> {
        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(self.output_extent)
            .build();

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let clear_values = &[color_clear_value];
        let begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffer)
            .render_area(render_area)
            .clear_values(clear_values)
            .build();

        let command_buffer = info.command_buffer;

        unsafe {
            self.device.cmd_begin_render_pass(
                command_buffer,
                &begin_info,
                vk::SubpassContents::INLINE,
            );

            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            self.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[self.descriptor_set],
                &[] as &[u32],
            );

            let push_buffer = PushConstantBuffer {
                divider_pos: info.divider_position,
                divider_width: info.divider_width as f32 / self.output_extent.width as f32,
                color: info.divider_color,
            };

            let bytes: &[u8] = bytemuck::bytes_of(&push_buffer);
            self.device.cmd_push_constants(
                command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                bytes,
            );

            self.device.cmd_draw(command_buffer, 3, 1, 0, 0);
            self.device.cmd_end_render_pass(command_buffer);
        }

        Ok(())
    }
}
