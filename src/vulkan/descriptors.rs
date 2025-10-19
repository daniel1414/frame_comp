use anyhow::Result;
use vulkanalia::prelude::v1_3::*;

/// This function should probably take in a descriptor type and the stage flags
/// for more flexibility. That's to be done when we will need descriptor set layouts
/// other than the one for the uniform buffer.
///
/// A descriptor set layout defines the structure of descriptors visible to shaders.
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

/// A descriptor is an object, that specifies how a shader accesses a resource.
/// It is metadata that tells Vulkan:
/// What resource to access (e.g., a uniform buffer, storage buffer, sampled image, etc.)
/// How to access (e.g., read-only, read-write, etc.)
///
/// Descriptor types:
///
/// UNIFORM_BUFFER: Used for UBOs like the MVP matrix.
/// STORAGE_BUFFER: Used for general-purpose storage buffers.
/// SAMPLED_IMAGE/COMBINED_IMAGE_SAMPLER: Used for sampled textures and their samplers.
/// STORAGE_IMAGE: Used for images that shaders can read from or write to directly.
///
/// Each descriptor is associated with a binding point in the shader (binding = n in the shader).
///
/// A descriptor set is a collection of descriptors grouped together. Represents a set of
/// resources that are made available to the shaders at the same time.
/// The sets are bound to the pipeline before issuing draw calls.
///
///
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
