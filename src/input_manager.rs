use winit::{
    event::{DeviceEvent, TouchPhase, WindowEvent},
    window,
};

#[derive(Debug, Default)]
pub struct InputManager {
    delta_x: f32,
    delta_y: f32,
    pub x: f64,
    pub y: f64,
    pub wx: f32,
    pub wy: f32,
    sensitivity: f32,
}

pub enum InputEvent<'a> {
    Window(&'a WindowEvent),
    Device(&'a DeviceEvent),
}

impl InputManager {
    pub fn process_events(&mut self, event: InputEvent) {
        use InputEvent::{Device, Window};
        match event {
            Device(device_event) => match device_event {
                DeviceEvent::MouseMotion { delta } => {
                    self.delta_x = delta.0 as f32;
                    self.delta_y = delta.1 as f32;
                }
                _ => (),
            },
            Window(window_event) => match window_event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.x = position.x;
                    self.y = position.y;
                    println!("Cursor position: x: {}, y: {}", position.x,position.y);
                }
                WindowEvent::MouseWheel { delta, phase, .. } => {
                    if *phase != TouchPhase::Ended {
                        let (wx, wy) = match delta {
                            winit::event::MouseScrollDelta::PixelDelta(delta) => {
                                (delta.x as f32, delta.y as f32)
                            }
                            winit::event::MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                        };
                        println!("wheel x: {}, wheel y: {}, phase: {:?}", self.wx, self.wy, phase);
                        self.wx += wx;
                        self.wy += wy;
                    }
                }
                _ => {
                    if *window_event != WindowEvent::RedrawRequested {
                        println!("Event: {:?}", window_event);
                    }
                }
            },
        }
    }
}
