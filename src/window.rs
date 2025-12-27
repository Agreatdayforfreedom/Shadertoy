use pollster::FutureExt;
use winit::{
    application::ApplicationHandler, event::{DeviceEvent, WindowEvent}, event_loop::ActiveEventLoop, window::{Window, WindowAttributes, WindowId}
};

use crate::{gpu::GpuState, input_manager::InputEvent};

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

        let mut attrs = WindowAttributes::default()
            .with_title("shader_toy");

        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::WindowAttributesExtWindows;
            attrs = attrs.with_class_name("myapp");
        }

        #[cfg(all(target_os = "linux", feature = "wayland"))]
        {
            use winit::platform::wayland::WindowAttributesExtWayland;
            attrs = attrs.with_name("myapp", "myapp");
        }
        let window = event_loop.create_window(attrs).unwrap();

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

        if !state.input(InputEvent::Window(&event)) {
            match event {
                WindowEvent::CloseRequested => {
                    println!("The close button was pressed; stopping");
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    let now = instant::Instant::now();
                    let dt = now - self.time;
                    self.time = now;

                    state.window().pre_present_notify();
                    state.window().request_redraw();
                    state.update(dt);
                    state.render();
                }
                _ => (),
            }
        }
    }
}
