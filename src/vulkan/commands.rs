use anyhow::Result;
use vulkanalia::prelude::v1_3::*;

pub(crate) fn begin_single_time_commands(
    device: &Device,
    command_pool: &vk::CommandPool,
) -> Result<vk::CommandBuffer> {
    let info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(*command_pool)
        .command_buffer_count(1);

    let command_buffer = unsafe { device.allocate_command_buffers(&info) }?[0];

    let info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe { device.begin_command_buffer(command_buffer, &info) }?;

    Ok(command_buffer)
}

pub(crate) fn end_single_time_commands(
    device: &Device,
    command_buffer: &vk::CommandBuffer,
    command_pool: &vk::CommandPool,
    queue: &vk::Queue,
) -> Result<()> {
    let command_buffers = &[*command_buffer];
    let info = vk::SubmitInfo::builder().command_buffers(command_buffers);

    unsafe {
        device.end_command_buffer(*command_buffer)?;

        device.queue_submit(*queue, &[info], vk::Fence::null())?;
        device.queue_wait_idle(*queue)?;

        device.free_command_buffers(*command_pool, command_buffers);
    }

    Ok(())
}
