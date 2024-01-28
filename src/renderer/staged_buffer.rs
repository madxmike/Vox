use std::{
    ops::{Deref},
    sync::Arc,
};

use vulkano::{
    buffer::{AllocateBufferError, Buffer, BufferContents, BufferCreateInfo, Subbuffer},
    command_buffer::CopyBufferInfo,
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
    Validated,
};

use super::vulkan::vulkan_renderer::VulkanRenderer;

pub struct StagedBuffer<T> {
    host_buffer: Subbuffer<[T]>,

    device_buffer: Subbuffer<[T]>,
}

impl<T> StagedBuffer<T>
where
    T: BufferContents,
{
    pub fn new_slice(
        allocator: Arc<dyn MemoryAllocator>,
        host_buffer_create_info: BufferCreateInfo,
        device_buffer_create_info: BufferCreateInfo,
        size: u64,
    ) -> Result<StagedBuffer<T>, Validated<AllocateBufferError>> {
        let host_buffer = Buffer::new_slice(
            allocator.clone(),
            host_buffer_create_info,
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            size,
        )?;

        let device_buffer = Buffer::new_slice(
            allocator.clone(),
            device_buffer_create_info,
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            size,
        )?;

        Ok(StagedBuffer {
            host_buffer,
            device_buffer,
        })
    }

    pub fn upload_to_device(&self, renderer: &VulkanRenderer) {
        renderer.immediate_submit(|cbb| {
            cbb.copy_buffer(CopyBufferInfo::buffers(
                self.host_buffer.clone(),
                self.device_buffer.clone(),
            ))
        })
    }

    pub fn device_buffer(&self) -> &Subbuffer<[T]> {
        &self.device_buffer
    }
}

impl<T> Deref for StagedBuffer<T>
where
    T: BufferContents,
{
    type Target = Subbuffer<[T]>;

    fn deref(&self) -> &Self::Target {
        &self.host_buffer
    }
}
