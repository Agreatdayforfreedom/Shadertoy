use pollster::FutureExt;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::gpu::GpuState;

pub struct App {
    time: instant::Instant,
    state: Option<GpuState>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            time: instant::Instant::now(),
            state: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();

        self.time = instant::Instant::now();
        let state = GpuState::new(window);

        self.state = Some(state.block_on());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = if let Some(state) = &mut self.state {
            state
        } else {
            panic!("NO state")
        };

        if !state.input(&event) {
            match event {
                WindowEvent::CloseRequested => {
                    println!("The close button was pressed; stopping");
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    let now = instant::Instant::now();
                    let dt = now - self.time;
                    self.time = now;
                    state.window().request_redraw();
                    state.update(dt);
                    state.render();
                }

                _ => (),
            }
        }
    }
}
