use std::time::Instant;

use taiko::{app::App, render::Renderer};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use taiko::{HEIGHT, WIDTH};

#[tokio::main]
async fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        .with_title("Taiko!!")
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let mut frame_time = Instant::now();

    let mut renderer = Renderer::new(window).await.unwrap();
    let mut app = App::new(&renderer).unwrap();

    event_loop.run(move |event, _, control_flow| {
        if !renderer.handle_event(&event) {
            match event {
                Event::WindowEvent { window_id, event } if window_id == renderer.window().id() => {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        } => {
                            control_flow.set_exit();
                        }

                        WindowEvent::Resized(size) => {
                            renderer.resize(size);
                        }

                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            renderer.resize(*new_inner_size);
                        }

                        _ => {}
                    }
                }

                Event::RedrawRequested(window_id) if window_id == renderer.window().id() => {
                    let time = Instant::now();
                    let delta = time.duration_since(frame_time).as_secs_f32();
                    frame_time = time;
                    app.update(delta, &renderer, control_flow);
                    match renderer.render(&mut app) {
                        Ok(_) => {}

                        Err(wgpu::SurfaceError::Lost) | Err(wgpu::SurfaceError::Outdated) => {
                            let size = renderer.size();
                            renderer.resize(*size);
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => control_flow.set_exit(),
                        Err(e) => log::error!("{e:?}"),
                    }
                }

                Event::MainEventsCleared => renderer.window().request_redraw(),

                _ => {}
            }
        }
    });
}
