mod camera;
mod cube_mesh;
mod mesh;
mod renderer;
mod shaders;
mod transform;
mod vulkan_renderer;

use std::f32::consts::PI;

use camera::Camera;
use renderer::Renderer;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use transform::Transform;
use vulkan_renderer::VulkanRenderer;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Hello World", 800, 600)
        .resizable()
        .vulkan()
        .build()
        .unwrap();
    let aspect_ratio =
        window.vulkan_drawable_size().0 as f32 / window.vulkan_drawable_size().1 as f32;

    let mut vulkan_renderer = VulkanRenderer::from_sdl_window(window);

    let mut event_pump = sdl_context.event_pump().unwrap();

    let camera = Camera {
        transform: Transform {
            position: glam::vec3(0.0, 0.0, 3.0),
            rotation: glam::Quat::from_euler(glam::EulerRot::XYZ, 36.7 * PI / 180.0, 0.0, 0.0),
        },
        near_clipping_plane: 0.01,
        far_clipping_plane: 100.0,
        field_of_view: 90.0,
        aspect_ratio,
    };

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

        vulkan_renderer.render(&camera);

        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}
