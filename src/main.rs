mod window_manager;
mod render_manager;
mod utils;

use winit::event_loop::{ControlFlow, EventLoop};
use window_manager::App;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("Error: {:?}", e);
    }
}
