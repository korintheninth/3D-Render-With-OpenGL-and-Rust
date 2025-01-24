use bytemuck::Contiguous;
use winit::application::ApplicationHandler;
use winit::event:: WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use crate::render_manager::RenderManager;

pub struct App {
    window: Option<Window>,
    render_manager: Option<RenderManager>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            render_manager: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("3D Window")
            .with_maximized(true);
        let window = event_loop.create_window(window_attributes).unwrap();
        self.render_manager = Some(RenderManager::new(&window));
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        static mut ZOOM: f64 = 1.0;
        static mut POS: (f64, f64) = (0.0, 0.0);
        static mut DIFF: (f64, f64) =  (0.0, 0.0);
        static mut PRESSED: bool =  false;
        
        let size = {
            let window = self.window.as_ref().unwrap();
            let size = window.inner_size();
            (size.width, size.height)
        };
        
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                if let Some(render_manager) = &self.render_manager {
                    
                    render_manager.render(size, unsafe {DIFF}, unsafe {ZOOM});
                    self.window.as_ref().unwrap().request_redraw();
                }
            }
            WindowEvent::MouseWheel { device_id:_, delta, phase:_} => {
                unsafe {
                    ZOOM += 0.05 * match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => y as f64,
                        winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y,
                    };
                }
            },
            WindowEvent::CursorMoved { device_id: _, position } => {
                unsafe {
                    if PRESSED {
                        DIFF = (DIFF.0 + position.x - POS.0, DIFF.1 + position.y - POS.1);
                    }
                    POS = (position.x, position.y);
                }
            }
            WindowEvent::MouseInput { device_id: _, state, button} => {
                if state == winit::event::ElementState::Pressed {
                    unsafe {
                        PRESSED = true;
                    }
                }
                else {
                    unsafe {
                        PRESSED = false;
                    }
                }
            }
            _ => (),
        }
    }
}