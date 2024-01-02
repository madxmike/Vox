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
    cube_mesh::CUBE_MESH,
    mesh::{Mesh, StaticMesh},
    renderer::Renderer,
    shaders::{
        self,
        default_lit::{self, DefaultLitIndex, DefaultLitVertex},
    },
    transform::Transform,
};

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

    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

    fixed_extent_render_context: Option<VulkanFixedExtentRenderContext>,
}
struct VulkanFixedExtentRenderContext {
    swapchain: Arc<Swapchain>,
    swapchain_images: Vec<Arc<Image>>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    queue: Arc<Queue>,

    active_graphics_pipeline: Arc<GraphicsPipeline>,
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

        let (physical_device, logical_device, queues) = create_devices(
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

        VulkanRenderer {
            sdl_window,
            vulkan_instance,
            vulkan_surface,
            physical_device,
            logical_device,
            queues,
            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
            fixed_extent_render_context: None,
        }
    }

    fn create_fixed_render_context(&mut self) -> VulkanFixedExtentRenderContext {
        let (swapchain, swapchain_images) = create_swapchain(
            &self.physical_device,
            &self.logical_device,
            &self.vulkan_surface,
        )
        .unwrap();
        let render_pass =
            create_render_pass(self.logical_device.clone(), swapchain.clone()).unwrap();
        let framebuffers =
            create_framebuffers(&self.memory_allocator, &swapchain_images, &render_pass).unwrap();

        // TODO (Michael): We should create all the pipelines we need to use once per context then just swap between them
        let vs = shaders::default_lit::vs::load(self.logical_device.clone()).unwrap();
        let fs = shaders::default_lit::fs::load(self.logical_device.clone()).unwrap();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [
                self.sdl_window.vulkan_drawable_size().0 as f32,
                self.sdl_window.vulkan_drawable_size().1 as f32,
            ],
            depth_range: 0.0..=1.0,
        };
        let graphics_pipeline = create_graphics_pipeline(
            self.logical_device.clone(),
            vs,
            fs,
            render_pass.clone(),
            viewport,
        )
        .unwrap();
        let queue = self.queues.next().unwrap();

        VulkanFixedExtentRenderContext {
            swapchain,
            swapchain_images,
            render_pass,
            framebuffers,
            active_graphics_pipeline: graphics_pipeline,
            queue,
        }
    }
}

impl Renderer for VulkanRenderer {
    fn render(&mut self, camera: &Camera, mesh: impl Mesh) {
        if let None = self.fixed_extent_render_context {
            self.fixed_extent_render_context = Some(self.create_fixed_render_context());
        }

        let mvp_buffer = SubbufferAllocator::new(
            self.memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );

        let render_context = &self.fixed_extent_render_context.as_ref().unwrap();

        let aspect_ratio = render_context.swapchain.image_extent()[0] as f32
            / render_context.swapchain.image_extent()[1] as f32;

        let mvp_buffer_subbuffer = {
            let (model, view, projection) = camera.mvp();

            let mvp_data = default_lit::vs::MVP {
                model: model.to_cols_array_2d(),
                view: view.to_cols_array_2d(),
                projection: projection.to_cols_array_2d(),
            };

            let subbuffer = mvp_buffer.allocate_sized().unwrap();
            *subbuffer.write().unwrap() = mvp_data;

            subbuffer
        };

        let descriptor_set_layout = render_context
            .active_graphics_pipeline
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

        let command_buffers = create_command_buffers(
            &self.memory_allocator,
            &self.command_buffer_allocator,
            &render_context.framebuffers,
            &render_context.queue,
            &render_context.active_graphics_pipeline,
            descriptor_set.clone(),
            mesh,
        );

        let (image_idx, _suboptimal, acquired_future) =
            swapchain::acquire_next_image(render_context.swapchain.clone(), None).unwrap();

        let _ = acquired_future
            .boxed()
            .then_execute(
                render_context.queue.clone(),
                command_buffers[image_idx as usize].clone(),
            )
            .unwrap()
            .then_swapchain_present(
                render_context.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    render_context.swapchain.clone(),
                    image_idx,
                ),
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

fn create_render_pass(
    logical_device: Arc<Device>,
    swapchain: Arc<Swapchain>,
) -> Result<Arc<RenderPass>, Validated<vulkano::VulkanError>> {
    vulkano::single_pass_renderpass!(
        logical_device,
        attachments: {
            color: {
                format: swapchain.image_format(),
                samples: 1,
                load_op: Clear,
                store_op: Store
            },
            depth: {
                format: Format::D16_UNORM,
                samples: 1,
                load_op: Clear,
                store_op: DontCare
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth},
        },
    )
}

fn create_framebuffers(
    memory_allocator: &Arc<StandardMemoryAllocator>,
    images: &[Arc<Image>],
    render_pass: &Arc<RenderPass>,
) -> Result<Vec<Arc<Framebuffer>>, Validated<vulkano::VulkanError>> {
    let extent = images[0].extent();
    let depth_stencil_image = Image::new(
        memory_allocator.clone(),
        ImageCreateInfo {
            format: Format::D16_UNORM,
            usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
            extent: extent,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
    )
    .unwrap();
    let depth_stencil_view = ImageView::new_default(depth_stencil_image.clone()).unwrap();

    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone())?;

            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view, depth_stencil_view.clone()],
                    ..Default::default()
                },
            )
        })
        .collect()
}

fn create_vertex_buffer(
    memory_allocator: &Arc<StandardMemoryAllocator>,
    verticies: Vec<DefaultLitVertex>,
) -> Result<Subbuffer<[DefaultLitVertex]>, Validated<AllocateBufferError>> {
    Buffer::from_iter(
        memory_allocator.clone(),
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

fn create_graphics_pipeline(
    logical_device: Arc<Device>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
    render_pass: Arc<RenderPass>,
    viewport: Viewport,
) -> Result<Arc<GraphicsPipeline>, Validated<vulkano::VulkanError>> {
    // TODO (Michael): Handle these vs / fs failures more gracefully
    let vs = vs.entry_point("main").unwrap();
    let fs = fs.entry_point("main").unwrap();

    let vertex_input_state =
        DefaultLitVertex::per_vertex().definition(&vs.info().input_interface)?;

    let stages = [
        PipelineShaderStageCreateInfo::new(vs),
        PipelineShaderStageCreateInfo::new(fs),
    ];

    let layout = PipelineLayout::new(
        logical_device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(logical_device.clone())
            .unwrap(), // TODO (Michael): Could we do something better here than just panicing?
    )?;

    // TODO (Michael): Better handle the None case
    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

    GraphicsPipeline::new(
        logical_device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState {
                ..Default::default()
            }),
            viewport_state: Some(ViewportState {
                viewports: [viewport].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState {
                cull_mode: CullMode::Back,
                front_face: FrontFace::Clockwise,
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState::simple()),
                ..Default::default()
            }),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
}

fn create_command_buffers(
    memory_allocator: &Arc<StandardMemoryAllocator>,
    command_buffer_allocator: &StandardCommandBufferAllocator,
    frame_buffers: &[Arc<Framebuffer>],
    queue: &Arc<Queue>,
    pipeline: &Arc<GraphicsPipeline>,
    descriptor_set: Arc<PersistentDescriptorSet>,
    mesh: impl Mesh,
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    let mesh_verticies = mesh.verticies();
    let mesh_normals = mesh.normals();

    let mut verticies: Vec<DefaultLitVertex> = vec![];
    for i in 0..mesh.verticies().len() {
        verticies.push(DefaultLitVertex {
            position: mesh_verticies[i].clone(),
            normal: mesh_normals[i].clone(),
        });
    }

    let vertex_buffer = create_vertex_buffer(&memory_allocator, verticies).unwrap();

    let index_buffer = Buffer::from_iter(
        memory_allocator.clone(),
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

    frame_buffers
        .iter()
        .map(|frame_buffer| {
            // TODO (Michael): Improve the error handling here
            let mut builder = AutoCommandBufferBuilder::primary(
                command_buffer_allocator,
                queue.queue_family_index(),
                CommandBufferUsage::MultipleSubmit,
            )
            .unwrap();

            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![
                            Some([0.0, 0.0, 0.0, 1.0].into()),
                            Some(format::ClearValue::Depth(1.0)),
                        ],
                        ..RenderPassBeginInfo::framebuffer(frame_buffer.clone())
                    },
                    SubpassBeginInfo {
                        ..Default::default()
                    },
                )
                .unwrap()
                .bind_pipeline_graphics(pipeline.clone())
                .unwrap()
                .bind_vertex_buffers(0, vertex_buffer.clone())
                .unwrap()
                .bind_descriptor_sets(
                    vulkano::pipeline::PipelineBindPoint::Graphics,
                    pipeline.layout().clone(),
                    0,
                    descriptor_set.clone(),
                )
                .unwrap()
                .bind_index_buffer(index_buffer.clone())
                .unwrap()
                .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)
                .unwrap()
                .end_render_pass(Default::default())
                .unwrap();

            builder.build().unwrap()
        })
        .collect()
}
