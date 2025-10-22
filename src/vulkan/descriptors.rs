use anyhow::Result;
use vulkanalia::prelude::v1_3::*;

pub(crate) fn create_descriptor_set_layout(device: &Device) -> Result<vk::DescriptorSetLayout> {
    let bindings = (0..2)
        .map(|i| {
            vk::DescriptorSetLayoutBinding::builder()
                .binding(i)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build()
        })
        .collect::<Vec<_>>();

    let info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(&bindings)
        .build();

    let descriptor_set_layout = unsafe { device.create_descriptor_set_layout(&info, None) }?;
    Ok(descriptor_set_layout)
}

pub(crate) fn create_descriptor_set(
    device: &Device,
    pool: &vk::DescriptorPool,
    layout: &vk::DescriptorSetLayout,
) -> Result<vk::DescriptorSet> {
    // We use the same layout for all swapchain images.
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(*pool)
        .set_layouts(std::slice::from_ref(layout))
        .build();

    let descriptor_sets = unsafe { device.allocate_descriptor_sets(&info) }?;

    Ok(descriptor_sets[0])
}

pub(crate) fn update_descriptor_sets(
    device: &Device,
    descriptor_set: &vk::DescriptorSet,
    sampler: &vk::Sampler,
    image_views: &[vk::ImageView; 2],
) {
    let infos = image_views
        .iter()
        .map(|image_view| {
            vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(*image_view)
                .sampler(*sampler)
                .build()
        })
        .collect::<Vec<_>>();

    let writes = infos
        .iter()
        .enumerate()
        .map(|(i, image_info)| {
            vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor_set)
                .dst_binding(i as u32)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(image_info))
                .build()
        })
        .collect::<Vec<_>>();

    // The second argument can be used to copy descriptor sets to each other.
    unsafe { device.update_descriptor_sets(&writes, &[] as &[vk::CopyDescriptorSet]) };
}
