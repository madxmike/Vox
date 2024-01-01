mod renderer;
mod shaders;
mod vulkan_renderer;

use renderer::Renderer;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use vulkan_renderer::VulkanRenderer;
use vulkano::buffer::BufferContents;

use vulkano::pipeline::graphics::vertex_input::Vertex;


fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Hello World", 800, 600)
        .resizable()
        .vulkan()
        .build()
        .unwrap();

    let mut vulkan_renderer = VulkanRenderer::from_sdl_window(window);

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

        vulkan_renderer.render();

        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}
