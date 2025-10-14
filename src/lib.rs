use anyhow::Result;
use std::rc::Rc;
use vulkanalia::prelude::v1_3::*;

use crate::vulkan::{
    buffers::{
        descriptors::create_descriptor_set,
        descriptors::{create_descriptor_pool, create_descriptor_set_layout},
    },
    pipeline::create_pipeline,
    render_pass::create_render_pass,
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
}

impl Drop for FrameComparator {
    fn drop(&mut self) {
        println!("Dropping the frame comparator!");
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.device
                .free_descriptor_sets(self.descriptor_pool, &[self.descriptor_set])
                .expect("Failed to destroy descriptor set. Somehing must've went wrong.");
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device.destroy_render_pass(self.render_pass, None);
        }
    }
}

impl FrameComparator {
    pub fn new(device: Rc<Device>, format: vk::Format, extent: vk::Extent2D) -> Result<Self> {
        let render_pass = create_render_pass(&device, format)?;
        let descriptor_pool = create_descriptor_pool(&device)?;
        let descriptor_set_layout = create_descriptor_set_layout(&device)?;
        let descriptor_set =
            create_descriptor_set(&device, &descriptor_pool, &descriptor_set_layout)?;

        let (pipeline_layout, pipeline) =
            create_pipeline(&device, &extent, &render_pass, &[descriptor_set_layout])?;

        Ok(Self {
            device,
            render_pass,
            descriptor_pool,
            descriptor_set_layout,
            descriptor_set,
            pipeline_layout,
            pipeline,
        })
    }

    pub fn compare(
        left_image: vk::Framebuffer,
        right_image: vk::Framebuffer,
        out_image: vk::Framebuffer,
    ) {
    }
}
