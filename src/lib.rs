use anyhow::Result;
use std::rc::Rc;
use vulkanalia::{
    prelude::v1_3::*,
    vk::{CommandBuffer, Rect2D},
};

use crate::vulkan::{
    buffers::descriptors::{
        create_descriptor_set, create_descriptor_set_layout, update_descriptor_sets,
    },
    pipeline::create_pipeline,
    render_pass::create_render_pass,
    sampler::create_image_sampler,
};

pub mod vulkan;

#[derive(Clone, Debug)]
pub struct FrameComparator {
    // For interfacing with the hardware,
    pub render_pass: vk::RenderPass,

    device: Rc<Device>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    extent: vk::Extent2D,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    sampler: vk::Sampler,
}

impl Drop for FrameComparator {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_sampler(self.sampler, None);
            self.device.destroy_pipeline(self.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
        }
    }
}

impl FrameComparator {
    pub fn new(
        device: Rc<Device>,
        graphics_queue: vk::Queue,
        command_pool: vk::CommandPool,
        descriptor_pool: vk::DescriptorPool,
        format: vk::Format,
        extent: vk::Extent2D,
        image_views: &[vk::ImageView; 2],
    ) -> Result<Self> {
        let render_pass = create_render_pass(&device, format)?;
        let descriptor_set_layout = create_descriptor_set_layout(&device)?;
        let descriptor_set =
            create_descriptor_set(&device, &descriptor_pool, &descriptor_set_layout)?;

        let (pipeline_layout, pipeline) =
            create_pipeline(&device, &extent, &render_pass, &[descriptor_set_layout])?;

        let sampler = create_image_sampler(&device)?;

        // updating descriptor sets
        update_descriptor_sets(&device, &descriptor_set, &sampler, image_views);

        // vertices and indices

        Ok(Self {
            render_pass,
            device,
            descriptor_pool,
            descriptor_set_layout,
            descriptor_set,
            pipeline_layout,
            pipeline,
            extent,
            graphics_queue,
            command_pool,
            sampler,
        })
    }

    pub fn render_pass(&self) -> vk::RenderPass {
        self.render_pass
    }
    pub fn compare(
        &self,
        command_buffer: CommandBuffer,
        percentage: f32,
        out_image: vk::Framebuffer,
    ) -> Result<()> {
        let render_area = Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(self.extent)
            .build();

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.02, 0.02, 0.02, 1.0],
            },
        };

        let clear_values = &[color_clear_value];
        let begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(out_image)
            .render_area(render_area)
            .clear_values(clear_values)
            .build();

        unsafe {
            println!("Begin comparator render pass");
            self.device.cmd_begin_render_pass(
                command_buffer,
                &begin_info,
                vk::SubpassContents::INLINE,
            );

            // Bind pipeline, descriptor sets, draw, end render pass and voila.
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
                std::slice::from_ref(&self.descriptor_set),
                &[] as &[u32],
            );

            // Push constants for the vertical divider (ideal for per-frame data)
            let bytes: &[u8] = bytemuck::bytes_of(&percentage);
            self.device.cmd_push_constants(
                command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                bytes,
            );

            self.device.cmd_draw(command_buffer, 3, 1, 0, 0);

            println!("End comparator render pass");
            self.device.cmd_end_render_pass(command_buffer);
        }

        Ok(())
    }
}
