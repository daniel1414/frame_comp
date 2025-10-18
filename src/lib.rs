use anyhow::Result;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use vulkanalia::prelude::v1_3::*;

use crate::vulkan::{
    descriptors::{create_descriptor_set, create_descriptor_set_layout, update_descriptor_sets},
    framebuffer::create_framebuffer,
    pipeline::create_pipeline,
    push_constants::PushConstantBuffer,
    render_pass::create_render_pass,
    sampler::create_image_sampler,
};

pub(crate) mod vulkan;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color(pub f32, pub f32, pub f32, pub f32);

pub struct FrameCompareInfo {
    command_buffer: vk::CommandBuffer, // The command buffer to record commands to.
    in_images: [vk::ImageView; 2],     // The incomming left and right image views.
    out_image: vk::ImageView,          // The output image
    divider_position: f32,             // Position of the divider in range [0.0; 1.0]
    divider_width: u8,                 // Width of the divider bar in logical pixels,
    divider_color: Color,              // Color of the divider in RGB
}

pub struct FrameCompareInfoBuilder {
    info: FrameCompareInfo,
}

impl FrameCompareInfoBuilder {
    pub fn command_buffer(mut self, buffer: vk::CommandBuffer) -> Self {
        self.info.command_buffer = buffer;
        self
    }

    pub fn out_image_view(mut self, image: vk::ImageView) -> Self {
        self.info.out_image = image;
        self
    }

    pub fn position(mut self, position: f32) -> Self {
        self.info.divider_position = position;
        self
    }

    pub fn width(mut self, width: u8) -> Self {
        self.info.divider_width = width;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.info.divider_color = color;
        self
    }

    pub fn build(self) -> FrameCompareInfo {
        self.info
    }
}

impl FrameCompareInfo {
    pub fn builder() -> FrameCompareInfoBuilder {
        FrameCompareInfoBuilder {
            info: FrameCompareInfo {
                command_buffer: vk::CommandBuffer::default(),
                in_images: [vk::ImageView::default(); 2],
                out_image: vk::ImageView::default(),
                divider_position: 0.5f32,
                divider_width: 5,
                divider_color: Color(0.0_f32, 0.0_f32, 0.0_f32, 1.0_f32),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct FrameComparator {
    // For interfacing with the hardware,
    pub render_pass: vk::RenderPass,

    device: Rc<Device>,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    extent: vk::Extent2D,
    sampler: vk::Sampler,
    framebuffers: RefCell<HashMap<vk::ImageView, vk::Framebuffer>>,
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
            self.sanitize_framebuffers();
        }
    }
}

impl FrameComparator {
    pub fn new(
        device: Rc<Device>,
        descriptor_pool: vk::DescriptorPool,
        format: vk::Format,
        extent: vk::Extent2D,
        image_views: &[vk::ImageView; 2],
        final_format: Option<vk::ImageLayout>,
    ) -> Result<Self> {
        let render_pass = create_render_pass(&device, format, final_format)?;
        let descriptor_set_layout = create_descriptor_set_layout(&device)?;
        let descriptor_set =
            create_descriptor_set(&device, &descriptor_pool, &descriptor_set_layout)?;

        let (pipeline_layout, pipeline) =
            create_pipeline(&device, &extent, &render_pass, &[descriptor_set_layout])?;

        let sampler = create_image_sampler(&device)?;

        // updating descriptor sets TODO: not here, in compare()
        update_descriptor_sets(&device, &descriptor_set, &sampler, image_views);

        Ok(Self {
            render_pass,
            device,
            descriptor_set_layout,
            descriptor_set,
            pipeline_layout,
            pipeline,
            extent,
            sampler,
            framebuffers: RefCell::new(HashMap::new()),
        })
    }

    pub fn render_pass(&self) -> vk::RenderPass {
        self.render_pass
    }

    fn sanitize_framebuffers(&self) {
        println!("Sanitizing framebuffers!");
        let mut cache = self.framebuffers.borrow_mut();
        for (_, framebuffer) in cache.drain() {
            unsafe {
                self.device.destroy_framebuffer(framebuffer, None);
            }
        }
    }

    pub fn compare(&self, info: &FrameCompareInfo) -> Result<()> {
        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(self.extent)
            .build();

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.02, 0.02, 0.02, 1.0],
            },
        };

        let clear_values = &[color_clear_value];

        if self.framebuffers.borrow().len() > 5 {
            self.sanitize_framebuffers();
        }

        let mut cache = self.framebuffers.borrow_mut();
        let framebuffer = *cache.entry(info.out_image).or_insert_with(|| {
            create_framebuffer(
                &self.device,
                &self.render_pass,
                &info.out_image,
                &self.extent,
            )
            .expect("failed to create framebuffer")
        });

        let begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(framebuffer)
            .render_area(render_area)
            .clear_values(clear_values)
            .build();

        let command_buffer = info.command_buffer;

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
            let push_buffer = PushConstantBuffer {
                divider_pos: info.divider_position,
                divider_width: info.divider_width as f32 / self.extent.width as f32,
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

            println!("End comparator render pass");
            self.device.cmd_end_render_pass(command_buffer);
        }

        Ok(())
    }
}
