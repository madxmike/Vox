use std::borrow::BorrowMut;
use std::sync::Arc;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::log;
use vulkano::buffer::{
    AllocateBufferError, Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer,
};
use vulkano::command_buffer::allocator::{CommandBufferAllocator, StandardCommandBufferAllocator};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
    SubpassBeginInfo,
};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::{PipelineDescriptorSetLayoutCreateInfo, PipelineLayoutCreateInfo};
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::{self, Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo};
use vulkano::swapchain::{
    self, Surface, SurfaceApi, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
};
use vulkano::sync::future::FenceSignalFuture;
use vulkano::sync::{GpuFuture, PipelineStage};
use vulkano::{Handle, Validated, VulkanError, VulkanLibrary, VulkanObject};

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct MyVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/vert.glsl",
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/frag.glsl",
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Hello World", 800, 600)
        .resizable()
        .vulkan()
        .build()
        .unwrap();

    // TODO (Michael): Enable validation features
    let mut instance_extensions =
        InstanceExtensions::from_iter(window.vulkan_instance_extensions().unwrap());

    let instance = Instance::new(VulkanLibrary::new().unwrap(), {
        let mut instance_info = InstanceCreateInfo::application_from_cargo_toml();
        instance_info.enabled_extensions = instance_extensions;
        instance_info
    })
    .unwrap();

    // TODO (Michael): Can we simplify this?
    let surface_handle = window
        .vulkan_create_surface(instance.handle().as_raw() as _)
        .unwrap();

    // SAFETY: Be sure not to drop the `window` before the `Surface` or vulkan `Swapchain`! (SIGSEGV otherwise)
    let surface = unsafe {
        Arc::new(Surface::from_handle(
            Arc::clone(&instance),
            <_ as Handle>::from_raw(surface_handle),
            SurfaceApi::Xlib,
            None,
        ))
    };

    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let (physical_device, logical_device, mut queues) =
        create_devices(&instance, &surface, device_extensions).unwrap();

    let (mut swapchain, images) =
        create_swapchain(&physical_device, &logical_device, &surface).unwrap();

    let render_pass = create_render_pass(logical_device.clone(), swapchain.clone()).unwrap();

    let frame_buffers = create_framebuffers(&images, &render_pass).unwrap();
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(logical_device.clone()));
    // TODO (Michael): Create pipeline

    let vs = vs::load(logical_device.clone()).unwrap();
    let fs = fs::load(logical_device.clone()).unwrap();

    let mut viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [
            window.vulkan_drawable_size().0 as f32,
            window.vulkan_drawable_size().1 as f32,
        ],
        depth_range: 0.0..=1.0,
    };

    let graphics_pipeline =
        create_graphics_pipeline(logical_device.clone(), vs, fs, render_pass, viewport).unwrap();

    let verticies = vec![
        MyVertex {
            position: [-0.5, 0.5],
        },
        MyVertex {
            position: [0.5, 0.5],
        },
        MyVertex {
            position: [0.0, -0.5],
        },
    ];

    let vertex_buffer = create_vertex_buffer(&memory_allocator, verticies).unwrap();
    let command_buffer_allocator =
        StandardCommandBufferAllocator::new(logical_device.clone(), Default::default());

    let queue = queues.next().unwrap();

    let command_buffers = create_command_buffers(
        &command_buffer_allocator,
        &frame_buffers,
        &queue,
        &graphics_pipeline,
        &vertex_buffer,
    );

    let mut event_pump = sdl_context.event_pump().unwrap();

    let (image_idx, suboptimal, acquired_future) =
        swapchain::acquire_next_image(swapchain.clone(), None).unwrap();

    let _ = acquired_future
        .boxed()
        .then_execute(queue.clone(), command_buffers[image_idx as usize].clone())
        .unwrap()
        .then_swapchain_present(
            queue.clone(),
            SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_idx),
        )
        .then_signal_fence_and_flush()
        .unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                _ => {}
            }
        }
        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn create_devices(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>,
    device_extensions: DeviceExtensions,
) -> Result<
    (
        Arc<vulkano::device::physical::PhysicalDevice>,
        Arc<vulkano::device::Device>,
        impl ExactSizeIterator + Iterator<Item = Arc<Queue>>,
    ),
    Validated<vulkano::VulkanError>,
> {
    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()?
        .filter(|physical_device| {
            physical_device
                .supported_extensions()
                .contains(&device_extensions)
        })
        .filter_map(|physical_device| {
            physical_device
                .queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.contains(QueueFlags::GRAPHICS)
                        && physical_device
                            .surface_support(i as u32, surface)
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
            enabled_extensions: device_extensions,
            ..Default::default()
        },
    )?;

    Ok((physical_device, logical_device, queues))
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
            image_usage: ImageUsage::COLOR_ATTACHMENT,
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
            }
        },
        pass: {
            color: [color],
            depth_stencil: {},
        },
    )
}

fn create_framebuffers(
    images: &[Arc<Image>],
    render_pass: &Arc<RenderPass>,
) -> Result<Vec<Arc<Framebuffer>>, Validated<vulkano::VulkanError>> {
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone())?;

            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
        })
        .collect()
}

fn create_vertex_buffer(
    memory_allocator: &Arc<StandardMemoryAllocator>,
    verticies: Vec<MyVertex>,
) -> Result<Subbuffer<[MyVertex]>, Validated<AllocateBufferError>> {
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

    let vertex_input_state = MyVertex::per_vertex().definition(&vs.info().input_interface)?;

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
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState {
                viewports: [viewport].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState::default()),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
}

fn create_command_buffers(
    allocator: &StandardCommandBufferAllocator,
    frame_buffers: &[Arc<Framebuffer>],
    queue: &Arc<Queue>,
    pipeline: &Arc<GraphicsPipeline>,
    vertex_buffer: &Subbuffer<[MyVertex]>,
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    frame_buffers
        .iter()
        .map(|frame_buffer| {
            // TODO (Michael): Improve the error handling here
            let mut builder = AutoCommandBufferBuilder::primary(
                allocator,
                queue.queue_family_index(),
                CommandBufferUsage::MultipleSubmit,
            )
            .unwrap();

            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
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
                .draw(vertex_buffer.len() as u32, 1, 0, 0)
                .unwrap()
                .end_render_pass(Default::default())
                .unwrap();

            builder.build().unwrap()
        })
        .collect()
}
