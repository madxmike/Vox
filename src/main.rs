mod camera;
mod cube_mesh;
mod mesh;
mod renderer;
mod shaders;
mod transform;
mod vulkan_renderer;
mod world;

use std::f32::consts::PI;
use std::time::SystemTime;

use camera::Camera;
use renderer::Renderer;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use sdl2::mouse::MouseButton;
use sdl2::sys::KeyCode;
use transform::Transform;
use vulkan_renderer::VulkanRenderer;
use world::{chunk, world_generation_system, world_render_system::WorldRenderSystem};

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

    let mut renderer = Box::new(VulkanRenderer::from_sdl_window(window)) as Box<dyn Renderer>;

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut camera = Camera {
        transform: Transform {
            position: glam::vec3(0.0, 0.0, 3.0),
            rotation: glam::Quat::from_euler(glam::EulerRot::XYZ, 34.7 * PI / 180.0, 0.0, 0.0),
        },
        near_clipping_plane: 0.01,
        far_clipping_plane: 100.0,
        field_of_view: 90.0,
        aspect_ratio,
    };

    let world = world_generation_system::generate_world(10);

    let timer_subsystem = sdl_context.timer().unwrap();
    let mut current_render_tick_time = timer_subsystem.performance_counter();
    let mut last_render_tick_time = current_render_tick_time.clone();
    let mut delta_time = 0.0;

    let camera_movement_speed = 5.0;
    let mut world_render_system = WorldRenderSystem::default();
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
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    camera.transform.position.z -= camera_movement_speed * delta_time;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    camera.transform.position.z += camera_movement_speed * delta_time;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    camera.transform.position.x -= camera_movement_speed * delta_time;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    camera.transform.position.x += camera_movement_speed * delta_time;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    camera.transform.position.y += camera_movement_speed * delta_time;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::LCtrl),
                    ..
                } => {
                    camera.transform.position.y -= camera_movement_speed * delta_time;
                }
                _ => {}
            }
        }

        last_render_tick_time = current_render_tick_time;
        current_render_tick_time = timer_subsystem.performance_counter();

        world_render_system
            .render_world(&mut renderer, &world, &camera)
            .unwrap();

        delta_time = ((current_render_tick_time - last_render_tick_time) as f32)
            / timer_subsystem.performance_frequency() as f32;

        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}
