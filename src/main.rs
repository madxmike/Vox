use std::sync::Arc;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::log;
use vulkano::buffer::{Buffer, BufferContents};
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::format::Format;
use vulkano::image::{Image, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::swapchain::{self, Surface, SurfaceApi, Swapchain, SwapchainCreateInfo};
use vulkano::{Handle, Validated, VulkanError, VulkanLibrary, VulkanObject};

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct MyVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Hello World", 800, 600)
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

    let (physical_device, logical_device, queues) =
        create_devices(&instance, &surface, device_extensions).expect("failed to create devices");

    let (swapchain, images) = create_swapchain(&physical_device, &logical_device, &surface)
        .expect("failed to create swapchain");

    // TODO (Michael): Create pipeline

    // TODO (Michael): Draw

    // TODO (Michael): Swapchain

    let mut event_pump = sdl_context.event_pump().unwrap();

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
