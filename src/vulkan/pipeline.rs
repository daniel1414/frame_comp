use anyhow::Result;
use vulkanalia::bytecode::Bytecode;
use vulkanalia::prelude::v1_3::*;

pub(crate) fn create_pipeline(
    device: &Device,
    extent: &vk::Extent2D,
    render_pass: &vk::RenderPass,
    descriptor_set_layouts: &[vk::DescriptorSetLayout],
) -> Result<(vk::PipelineLayout, vk::Pipeline)> {
    let vert = include_bytes!("shaders/vert.spv");
    let frag = include_bytes!("shaders/frag.spv");

    let vert_module = create_shader_module(device, vert)?;
    let frag_module = create_shader_module(device, frag)?;

    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_module)
        .name(b"main\0")
        .build();

    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_module)
        .name(b"main\0")
        .build();

    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false)
        .build();

    // Area of the framebuffer to render to. In our case the whole area.
    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(extent.width as f32)
        .height(extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0)
        .build();

    // Area of the framebuffer that fragments are allowed to affect. In our case the whole area.
    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(*extent)
        .build();

    let viewports = &[viewport];
    let scissors = &[scissor];

    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(viewports)
        .scissors(scissors)
        .build();

    // The rasterization state divides polygons into fragments (which end up being pixels on the screen)
    // and performs fragment culling - removing fragments that don't make it into the view.
    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false)
        .build();

    let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(true)
        .min_sample_shading(0.2)
        .rasterization_samples(vk::SampleCountFlags::_1)
        .build();

    let attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .blend_enable(false)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD)
        .build();

    let attachments = &[attachment];

    // Blending new fragments with the existing ones in the framebuffer.
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0])
        .build();

    // The pipeline layout is like a blueprint that defines:
    // 1. Descriptor sets: How resources like textures and uniform buffers are accessed
    //    by the shaders.
    // 2. Push constants: Small amounts of data sent to shaders for per-draw customization.
    // One push constant for the vertival divider.
    let push_constant_range = vk::PushConstantRange::builder()
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .offset(0)
        .size(std::mem::size_of::<f32>() as u32)
        .build();

    let set_layouts = descriptor_set_layouts;
    let layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(set_layouts)
        .push_constant_ranges(std::slice::from_ref(&push_constant_range))
        .build();

    let pipeline_layout = unsafe { device.create_pipeline_layout(&layout_info, None) }?;

    /* let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
    // Specifies if the depth of new fragments should be compared to the depth buffer
    // to see if they should be discarded.
    .depth_test_enable(true)
    // Specifies if the new depth of fragments that pass the depth test should actually
    // be written to the depth buffer.
    .depth_write_enable(true)
    // Comparison that is performed to keep or discard fragments. For us lower depth = closer,
    // So the depth of new fragments should be less.
    .depth_compare_op(vk::CompareOp::LESS)
    // These three parameters are used for the optional depth bound test. This allows to
    // only keep fragments that fall within the specified depth range. This is optional.
    //.depth_bounds_test_enable(true)
    //.min_depth_bounds(0.0)
    //.max_depth_bounds(1.0)
    .depth_bounds_test_enable(false)
    // We will not be using stencil operations here.
    .stencil_test_enable(false); */

    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&[] as &[vk::VertexInputBindingDescription])
        .vertex_attribute_descriptions(&[] as &[vk::VertexInputAttributeDescription])
        .build();

    let stages = &[vert_stage, frag_stage];
    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .color_blend_state(&color_blend_state)
        .layout(pipeline_layout)
        //.depth_stencil_state(&depth_stencil_state)
        // Link this pipeline to the correct render pass.
        .render_pass(*render_pass)
        // And the right subpass.
        .subpass(0)
        .build();

    let pipeline = unsafe {
        let pipeline = device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)?
            .0[0];

        device.destroy_shader_module(vert_module, None);
        device.destroy_shader_module(frag_module, None);
        pipeline
    };

    Ok((pipeline_layout, pipeline))
}

fn create_shader_module(device: &Device, bytecode: &[u8]) -> Result<vk::ShaderModule> {
    let bytecode = Bytecode::new(bytecode).unwrap();
    let info = vk::ShaderModuleCreateInfo::builder()
        .code_size(bytecode.code_size())
        .code(bytecode.code());

    let module = unsafe { device.create_shader_module(&info, None) }?;
    Ok(module)
}
