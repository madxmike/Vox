use std::{
    sync::Arc,
};

use sdl2::video::Window;
use vulkano::{
    buffer::{
        Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer,
    },
    command_buffer::{allocator::StandardCommandBufferAllocator, CommandBufferExecFuture},
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator,
    },
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags,
    },
    image::Image,
    instance::{Instance, InstanceCreateInfo, InstanceExtensions},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    swapchain::{
        self, PresentFuture, PresentMode, Surface, SurfaceApi, Swapchain, SwapchainAcquireFuture,
        SwapchainCreateInfo, SwapchainPresentInfo,
    },
    sync::{
        self,
        future::{FenceSignalFuture, JoinFuture},
        GpuFuture,
    }, Handle, Validated, VulkanLibrary, VulkanObject,
};

use super::{
    default_lit_pipeline::{DefaultLitPipeline, MeshVertex},
    mvp::MVP,
};

const REQUIRED_DEVICE_EXTENSIONS: DeviceExtensions = DeviceExtensions {
    khr_swapchain: true,
    ..DeviceExtensions::empty()
};

const REQUIRED_DEVICE_FEATURES: Features = Features {
    fill_mode_non_solid: true,
    ..Features::empty()
};

pub struct VulkanRenderer {
    sdl_window: Window,
    vulkan_instance: Arc<Instance>,
    vulkan_surface: Arc<Surface>,
    physical_device: Arc<PhysicalDevice>,
    logical_device: Arc<Device>,
    queues: Box<dyn ExactSizeIterator<Item = Arc<Queue>>>,
    queue: Arc<Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

    swapchain: Arc<Swapchain>,
    swapchain_images: Vec<Arc<Image>>,
    pipelines: Pipelines,
    fences: Vec<
        Option<
            Arc<
                FenceSignalFuture<
                    PresentFuture<
                        CommandBufferExecFuture<
                            JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>,
                        >,
                    >,
                >,
            >,
        >,
    >,
    last_fence_index: usize,
}

pub struct Pipelines {
    default_lit: DefaultLitPipeline,
}

impl VulkanRenderer {
    pub fn from_sdl_window(sdl_window: Window) -> VulkanRenderer {
        let _library = VulkanLibrary::new().unwrap();

        // TODO (Michael): Enable validation features
        let instance_extensions =
            InstanceExtensions::from_iter(sdl_window.vulkan_instance_extensions().unwrap());

        let vulkan_instance = Instance::new(VulkanLibrary::new().unwrap(), {
            let mut instance_info = InstanceCreateInfo::application_from_cargo_toml();
            instance_info.enabled_extensions = instance_extensions;
            instance_info
        })
        .unwrap();

        // TODO (Michael): Can we simplify this?
        let surface_handle = sdl_window
            .vulkan_create_surface(vulkan_instance.handle().as_raw() as _)
            .unwrap();

        // SAFETY: Be sure not to drop the `window` before the `Surface` or vulkan `Swapchain`! (SIGSEGV otherwise)
        let vulkan_surface = unsafe {
            Arc::new(Surface::from_handle(
                Arc::clone(&vulkan_instance),
                <_ as Handle>::from_raw(surface_handle),
                SurfaceApi::Xlib,
                None,
            ))
        };

        let (physical_device, logical_device, mut queues) = VulkanRenderer::create_devices(
            &vulkan_instance,
            &vulkan_surface,
            REQUIRED_DEVICE_EXTENSIONS,
            REQUIRED_DEVICE_FEATURES,
        )
        .unwrap();

        let memory_allocator =
            Arc::new(StandardMemoryAllocator::new_default(logical_device.clone()));
        let command_buffer_allocator =
            StandardCommandBufferAllocator::new(logical_device.clone(), Default::default());
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            logical_device.clone(),
            Default::default(),
        ));

        let extent = [
            sdl_window.vulkan_drawable_size().0 as f32,
            sdl_window.vulkan_drawable_size().1 as f32,
        ];
        let (swapchain, swapchain_images, pipelines) =
            VulkanRenderer::window_extent_dependent_setup(
                &logical_device,
                &physical_device,
                &vulkan_surface,
                extent,
            );
        let queue = queues.next().unwrap();
        let images_len = swapchain_images.len();

        VulkanRenderer {
            sdl_window,
            vulkan_instance,
            vulkan_surface,
            physical_device,
            logical_device,
            queues,
            queue,
            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
            swapchain: swapchain,
            swapchain_images: swapchain_images,
            pipelines: pipelines,
            fences: vec![None; images_len],
            last_fence_index: 0,
        }
    }

    fn window_extent_dependent_setup<'a>(
        device: &Arc<Device>,
        physical_device: &Arc<PhysicalDevice>,
        vulkan_surface: &Arc<Surface>,
        extent: [f32; 2],
    ) -> (Arc<Swapchain>, Vec<Arc<Image>>, Pipelines) {
        let capabilities = physical_device
            .surface_capabilities(vulkan_surface, Default::default())
            .unwrap();

        let surface_formats = physical_device
            .surface_formats(vulkan_surface, Default::default())
            .unwrap();
        let (image_format, color_space) = surface_formats.get(0).unwrap();

        let (swapchain, swapchain_images) = Swapchain::new(
            device.clone(),
            vulkan_surface.clone(),
            SwapchainCreateInfo {
                min_image_count: capabilities.min_image_count,
                image_format: image_format.to_owned(),
                image_color_space: color_space.to_owned(),
                image_extent: capabilities.max_image_extent,
                image_array_layers: capabilities.max_image_array_layers,
                image_usage: capabilities.supported_usage_flags,
                composite_alpha: capabilities
                    .supported_composite_alpha
                    .into_iter()
                    .next()
                    .unwrap()
                    .to_owned(),
                present_mode: PresentMode::Immediate,
                ..Default::default()
            },
        )
        .unwrap();

        let default_lit_pipeline =
            DefaultLitPipeline::new(device, &swapchain, &swapchain_images, extent).unwrap();

        let pipelines = Pipelines {
            default_lit: default_lit_pipeline,
        };

        (swapchain, swapchain_images, pipelines)
    }

    fn create_devices(
        vulkan_instance: &Arc<Instance>,
        vulkan_surface: &Arc<Surface>,
        required_device_extensions: DeviceExtensions,
        required_device_features: Features,
    ) -> Result<
        (
            Arc<PhysicalDevice>,
            Arc<Device>,
            Box<dyn ExactSizeIterator<Item = Arc<Queue>>>,
        ),
        Validated<vulkano::VulkanError>,
    > {
        let (physical_device, queue_family_index) = vulkan_instance
            .enumerate_physical_devices()?
            .filter(|physical_device| {
                physical_device
                    .supported_extensions()
                    .contains(&required_device_extensions)
            })
            .filter_map(|physical_device| {
                physical_device
                    .queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
                            && physical_device
                                .surface_support(i as u32, vulkan_surface)
                                .unwrap_or(false)
                    })
                    .map(|q| (physical_device, q as u32))
            })
            .min_by_key(
                |(physical_device, _)| match physical_device.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 0,
                    PhysicalDeviceType::IntegratedGpu => 1,
                    PhysicalDeviceType::VirtualGpu => 2,
                    PhysicalDeviceType::Cpu => 3,

                    // Note that there exists `PhysicalDeviceType::Other`, however,
                    // `PhysicalDeviceType` is a non-exhaustive enum. Thus, one should
                    // match wildcard `_` to catch all unknown device types.
                    _ => 4,
                },
            )
            .ok_or(vulkano::VulkanError::ExtensionNotPresent)?;

        let (logical_device, queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: required_device_extensions,
                enabled_features: required_device_features,
                ..Default::default()
            },
        )?;

        Ok((physical_device, logical_device, Box::new(queues)))
    }

    pub fn create_vertex_buffer<T, I>(
        &mut self,
        verticies: I,
    ) -> Result<vulkano::buffer::Subbuffer<[T]>, Validated<vulkano::buffer::AllocateBufferError>>
    where
        T: BufferContents,
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            verticies,
        )
    }

    pub fn create_sized_vertex_buffer<T>(
        &self,
        bytes: u64,
    ) -> Result<vulkano::buffer::Subbuffer<[T]>, Validated<vulkano::buffer::AllocateBufferError>>
    where
        T: BufferContents,
    {
        Buffer::new_slice(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            bytes,
        )
    }

    pub fn create_index_buffer<T, I>(
        &mut self,
        indicies: I,
    ) -> Result<vulkano::buffer::Subbuffer<[T]>, Validated<vulkano::buffer::AllocateBufferError>>
    where
        T: BufferContents,
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indicies,
        )
    }

    pub fn create_sized_index_buffer<T>(
        &self,
        bytes: u64,
    ) -> Result<vulkano::buffer::Subbuffer<[T]>, Validated<vulkano::buffer::AllocateBufferError>>
    where
        T: BufferContents,
    {
        Buffer::new_slice(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            bytes,
        )
    }

    pub fn default_lit(
        &mut self,
        mvp: MVP,
        vertex_buffer: &Subbuffer<[MeshVertex]>,
        index_buffer: &Subbuffer<[u32]>,
    ) {
        let descriptor_set = self
            .pipelines
            .default_lit
            .create_descriptor_set(&self.memory_allocator, &self.descriptor_set_allocator, mvp)
            .unwrap();

        // TODO (Michael) call pipeline
        let command_buffers = self
            .pipelines
            .default_lit
            .create_command_buffers(
                &self.command_buffer_allocator,
                self.queue.clone(),
                vertex_buffer,
                index_buffer,
                descriptor_set,
            )
            .unwrap();

        let (image_idx, _suboptimal, acquired_future) =
            swapchain::acquire_next_image(self.swapchain.clone(), None).unwrap();

        if let Some(image_fence) = &mut self.fences[image_idx as usize] {
            image_fence.wait(None).unwrap();
        }

        let previous_future = match self.fences[self.last_fence_index].clone() {
            // Create a NowFuture
            None => {
                let mut now = sync::now(self.logical_device.clone());
                now.cleanup_finished();
                now.boxed()
            }
            // Use the existing FenceSignalFuture
            Some(fence) => fence.boxed(),
        };

        let future = previous_future
            .join(acquired_future)
            .then_execute(
                self.queue.clone(),
                command_buffers[image_idx as usize].clone(),
            )
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_idx),
            )
            .then_signal_fence_and_flush()
            .unwrap();

        self.fences[image_idx as usize] = Some(Arc::new(future));
        self.last_fence_index = image_idx as usize;
    }
}
