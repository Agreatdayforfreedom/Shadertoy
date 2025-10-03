use window::App;
use winit::event_loop::{ControlFlow, EventLoop};

mod gpu;
mod quad;
mod sprite;
mod test;
mod texture;
mod uniform;
mod window;
mod uniforms;

pub fn run() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();

    
    event_loop.run_app(&mut app).unwrap();
}
