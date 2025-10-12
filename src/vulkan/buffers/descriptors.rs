use anyhow::Result;
use vulkanalia::prelude::v1_3::*;

pub type Mat4 = cgmath::Matrix4<f32>;

/// This function should probably take in a descriptor type and the stage flags
/// for more flexibility. That's to be done when we will need descriptor set layouts
/// other than the one for the uniform buffer.
///
/// A descriptor set layout defines the structure of descriptors visible to shaders.
pub fn create_descriptor_set_layout(device: &Device) -> Result<vk::DescriptorSetLayout> {
    let left_sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .build();

    let right_sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .build();

    let bindings = &[left_sampler_binding, right_sampler_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(bindings)
        .build();

    let descriptor_set_layout = unsafe { device.create_descriptor_set_layout(&info, None) }?;

    Ok(descriptor_set_layout)
}

/// A descriptor pool is an object that manages the memory required for allocating descriptor sets.
/// Pools alow efficient batch allocation and destruction of descriptor sets.
pub fn create_descriptor_pool(device: &Device) -> Result<vk::DescriptorPool> {
    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(2)
        .build();

    let pool_sizes = &[sampler_size];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(1)
        .build();

    let descriptor_pool = unsafe { device.create_descriptor_pool(&info, None) }?;

    Ok(descriptor_pool)
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
pub fn create_descriptor_sets(device: &Device, pool: &vk::DescriptorPool, layout: &vk::DescriptorSetLayout) -> Result<Vec<vk::DescriptorSet>> {
    // We use the same layout for all swapchain images.
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(*pool)
        .set_layouts(std::slice::from_ref(layout))
        .build();

    let descriptor_sets = unsafe { device.allocate_descriptor_sets(&info) }?;

    let image_info = vk::DescriptorImageInfo::builder()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(data.texture_image_view)
        .sampler(data.texture_sampler)
        .build();

    let image_infos = &[image_info];

    let sampler_write = vk::WriteDescriptorSet::builder()
        .dst_set(descriptor_sets[0])
        .dst_binding(1)
        .dst_array_element(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(image_infos);

    // The second argument can be used to copy descriptor sets to each other.
    unsafe {
        device.update_descriptor_sets(
            &[sampler_write],
            &[] as &[vk::CopyDescriptorSet],
        )
    };

    Ok(descriptor_sets)
}
