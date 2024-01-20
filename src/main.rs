mod camera;
mod renderer;
mod transform;
mod world;
use std::{f32::consts::PI, ops::Deref};

use camera::Camera;
use renderer::vulkan::vulkan_renderer::VulkanRenderer;
use renderer::world_render_system::WorldRenderSystem;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use transform::Transform;
use world::{
    block::Block,
    block_position::BlockPosition,
    chunk,
    world_generation_system::{self, WorldGenerationSettings},
};

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

    let mut renderer = Box::new(VulkanRenderer::from_sdl_window(window));

    let mut event_pump = sdl_context.event_pump().unwrap();

    sdl_context.mouse().set_relative_mouse_mode(true);
    let mut camera = Camera {
        transform: Transform::new(glam::vec3(0.0, 0.0, 3.0), glam::vec3(0.0, 0.0, 0.0)),
        local_transform: Transform::default(),
        near_clipping_plane: 0.01,
        far_clipping_plane: 1000.0,
        field_of_view: 90.0,
        aspect_ratio,

        ..Camera::default()
    };

    let mut world = world_generation_system::generate_world(
        10,
        WorldGenerationSettings {
            max_width: 10,
            max_height: 10,
            max_length: 10,
        },
    );

    let timer_subsystem = sdl_context.timer().unwrap();
    let mut current_render_tick_time = timer_subsystem.performance_counter();
    let mut last_render_tick_time = current_render_tick_time.clone();
    let mut delta_time = 0.0;

    let camera_movement_speed = 250.0;
    let mut world_render_system = WorldRenderSystem::new(&renderer);
    let mut flip = 0;
    world_render_system.build_chunk_meshes(&world);
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
                } => camera.r#move(0.0, 0.0, -camera_movement_speed * delta_time),
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => camera.r#move(0.0, 0.0, camera_movement_speed * delta_time),
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => camera.r#move(-camera_movement_speed * delta_time, 0.0, 0.0),
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => camera.r#move(camera_movement_speed * delta_time, 0.0, 0.0),
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    camera.transform.translate_along_axis(
                        transform::Axis::Up,
                        camera_movement_speed * delta_time,
                    );
                }
                Event::KeyDown {
                    keycode: Some(Keycode::LCtrl),
                    ..
                } => {
                    camera.transform.translate_along_axis(
                        transform::Axis::Up,
                        -camera_movement_speed * delta_time,
                    );
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Minus),
                    ..
                } => {
                    let eurler = camera.transform.rotation.to_euler(glam::EulerRot::XYZ);
                    camera.transform.rotation = glam::Quat::from_euler(
                        glam::EulerRot::XYZ,
                        eurler.0,
                        eurler.1 + PI,
                        eurler.2,
                    );
                }
                Event::KeyDown {
                    keycode: Some(Keycode::KpPlus),
                    ..
                } => {
                    let eurler = camera.transform.rotation.to_euler(glam::EulerRot::XYZ);
                    camera.transform.rotation =
                        glam::Quat::from_euler(glam::EulerRot::XYZ, -eurler.0, eurler.1, eurler.2);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::KpEnter),
                    ..
                } => {
                    camera.transform.position = glam::Vec3::default();
                }

                Event::MouseMotion { xrel, yrel, .. } => {
                    camera.rotate_yaw((xrel as f32 * delta_time * 10.0) * PI / 180.0);
                    camera.rotate_pitch((yrel as f32 * delta_time * 10.0) * PI / 180.0)
                }
                _ => {}
            }
        }

        last_render_tick_time = current_render_tick_time;
        current_render_tick_time = timer_subsystem.performance_counter();

        world_render_system.render_world(&mut renderer, &world, &camera);

        delta_time = ((current_render_tick_time - last_render_tick_time) as f32)
            / timer_subsystem.performance_frequency() as f32;
        // dbg!(delta_time);
    }
}
