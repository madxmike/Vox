use std::{
    f32::consts::{FRAC_PI_2, PI},
    os::raw,
    sync::Arc,
};

use glam::Mat4;
use sdl2::video::Window;
use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        AllocateBufferError, Buffer, BufferCreateInfo, BufferUsage, IndexBuffer, Subbuffer,
    },
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo,
    },
    descriptor_set::{
        self, allocator::StandardDescriptorSetAllocator, layout::DescriptorSetLayout,
        DescriptorSet, DescriptorSetsCollection, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
    },
    format::{self, Format},
    image::{
        view::{ImageView, ImageViewCreateInfo},
        Image, ImageCreateInfo, ImageUsage,
    },
    instance::{Instance, InstanceCreateInfo, InstanceExtensions},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{self, DepthState, DepthStencilState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, RasterizationState},
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        GraphicsPipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    shader::ShaderModule,
    swapchain::{self, Surface, SurfaceApi, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo},
    sync::GpuFuture,
    Handle, Validated, VulkanLibrary, VulkanObject,
};

use crate::{
    camera::Camera,
    mesh::Mesh,
    renderer::renderer::{self, Renderer},
    shaders::{self},
    transform::Transform,
};

use super::default_lit_pipeline::{self, vs, DefaultLitPipeline, DefaultLitVertex};

const REQUIRED_DEVICE_EXTENSIONS: DeviceExtensions = DeviceExtensions {
    khr_swapchain: true,
    ..DeviceExtensions::empty()
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

        let (physical_device, logical_device, mut queues) = create_devices(
            &vulkan_instance,
            &vulkan_surface,
            REQUIRED_DEVICE_EXTENSIONS,
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
        }
    }

    fn window_extent_dependent_setup<'a>(
        device: &Arc<Device>,
        physical_device: &Arc<PhysicalDevice>,
        vulkan_surface: &Arc<Surface>,
        extent: [f32; 2],
    ) -> (Arc<Swapchain>, Vec<Arc<Image>>, Pipelines) {
        let (swapchain, swapchain_images) =
            create_swapchain(physical_device, device, vulkan_surface).unwrap();

        let default_lit_pipeline =
            DefaultLitPipeline::new(device, &swapchain, &swapchain_images, extent).unwrap();

        let pipelines = Pipelines {
            default_lit: default_lit_pipeline,
        };

        (swapchain, swapchain_images, pipelines)
    }
}

impl Renderer for VulkanRenderer {
    fn render(&mut self, camera: &Camera, mesh: Box<dyn Mesh>) {
        let mvp_buffer = SubbufferAllocator::new(
            self.memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );

        let mvp_buffer_subbuffer = {
            let (model, view, projection) = camera.mvp();

            let mvp_data = vs::MVP {
                model: model.to_cols_array_2d(),
                view: view.to_cols_array_2d(),
                projection: projection.to_cols_array_2d(),
            };

            let subbuffer = mvp_buffer.allocate_sized().unwrap();
            *subbuffer.write().unwrap() = mvp_data;

            subbuffer
        };

        let descriptor_set_layout = self
            .pipelines
            .default_lit
            .layout()
            .set_layouts()
            .get(0)
            .unwrap();
        let descriptor_set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator.clone(),
            descriptor_set_layout.clone(),
            [WriteDescriptorSet::buffer(0, mvp_buffer_subbuffer)],
            [],
        )
        .unwrap();

        let mesh_verticies = mesh.verticies();
        let mesh_normals = mesh.normals();

        let mut verticies: Vec<DefaultLitVertex> = vec![];
        let normal_idx = 0;
        for i in 0..mesh.verticies().len() {
            verticies.push(DefaultLitVertex {
                position: mesh_verticies[i].to_array(),
                normal: mesh_normals[i / 4].to_array(),
            });
        }

        let vertex_buffer = Buffer::from_iter(
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
        .unwrap();

        let index_buffer = Buffer::from_iter(
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
            mesh.indicies().to_owned().iter().map(|index| *index),
        )
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

        let _ = acquired_future
            .boxed()
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
    }
}

fn create_swapchain(
    physical_device: &Arc<PhysicalDevice>,
    logical_device: &Arc<Device>,
    surface: &Arc<Surface>,
) -> Result<(Arc<Swapchain>, Vec<Arc<Image>>), Validated<vulkano::VulkanError>> {
    let capabilities = physical_device.surface_capabilities(surface, Default::default())?;

    let surface_formats = physical_device.surface_formats(surface, Default::default())?;
    let (image_format, color_space) = surface_formats.get(0).unwrap();

    let (swapchain, images) = Swapchain::new(
        logical_device.clone(),
        surface.clone(),
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
            ..Default::default()
        },
    )?;

    Ok((swapchain, images))
}

fn create_devices(
    vulkan_instance: &Arc<Instance>,
    vulkan_surface: &Arc<Surface>,
    required_device_extensions: DeviceExtensions,
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
            ..Default::default()
        },
    )?;

    Ok((physical_device, logical_device, Box::new(queues)))
}
