use std::sync::Arc;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};
use vulkano::swapchain::{Surface, SurfaceApi};
use vulkano::{VulkanLibrary, VulkanObject, Handle};

fn main() {

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Hello World", 800, 600)
        .vulkan()
        .build()
        .unwrap();

    let instance_extensions = InstanceExtensions::from_iter(window.vulkan_instance_extensions().unwrap());

    let _instance = Instance::new(VulkanLibrary::new().unwrap(), {
        let mut instance_info = InstanceCreateInfo::application_from_cargo_toml();
        instance_info.enabled_extensions = instance_extensions;
        instance_info
    }).unwrap();

    let surface_handle = window
        .vulkan_create_surface(_instance.handle().as_raw() as _)
        .unwrap();
    
    // SAFETY: Be sure not to drop the `window` before the `Surface` or vulkan `Swapchain`! (SIGSEGV otherwise)
    let surface = unsafe {
        Surface::from_handle(
            Arc::clone(&_instance),
            <_ as Handle>::from_raw(surface_handle),
            SurfaceApi::Xlib,
            None,
        )
    };

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
