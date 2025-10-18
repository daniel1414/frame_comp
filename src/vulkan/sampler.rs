use anyhow::Result;
use vulkanalia::prelude::v1_3::*;

pub(crate) fn create_image_sampler(device: &Device) -> Result<vk::Sampler> {
    let sampler_create_info = vk::SamplerCreateInfo::builder()
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .anisotropy_enable(false)
        .min_filter(vk::Filter::LINEAR)
        .mag_filter(vk::Filter::LINEAR)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .build();

    let sampler = unsafe { device.create_sampler(&sampler_create_info, None)? };
    Ok(sampler)
}
