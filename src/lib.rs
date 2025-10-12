use anyhow::Result;
use vulkanalia::prelude::v1_3::*;
use std::rc::Rc;

use crate::vulkan::{buffers::uniform_buffer::{create_descriptor_pool, create_descriptor_set_layout}, pipeline::create_pipeline, render_pass::create_render_pass};

pub mod vulkan;

#[derive(Clone, Debug)]
pub struct FrameComparator {
    // For interfacing with the hardware,
    device: Rc<Device>,
    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

impl FrameComparator {
    pub fn new(
        device: Rc<Device>,
        format: vk::Format,
        extent: vk::Extent2D,
    ) -> Result<Self> {
        let render_pass = create_render_pass(&device, format)?;
        let descriptor_pool = create_descriptor_pool(&device)?;
        let descriptor_set_layout = create_descriptor_set_layout(&device)?;
        let (pipeline_layout, pipeline) = create_pipeline(&device, &extent, &render_pass,&[descriptor_set_layout])?;

        Ok(Self { device, render_pass, pipeline_layout, pipeline })
    }

    pub fn get_render_pass(&self) -> vk::RenderPass {
        self.render_pass
    }

    pub fn compare(left_image: vk::Framebuffer, right_image: vk::Framebuffer, out_image: vk::Framebuffer) {

    }


}

