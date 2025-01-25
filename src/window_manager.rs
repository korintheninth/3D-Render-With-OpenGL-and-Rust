use winit::application::ApplicationHandler;
use winit::event:: WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::Key;
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
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
            .with_visible(false);
        let window = event_loop.create_window(window_attributes).unwrap();
        self.render_manager = Some(RenderManager::new(&window));
        window.set_visible(true);
        window.set_maximized(true);
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        static mut ZOOM: f64 = 1.0;
        static mut POS: (f64, f64) = (0.0, 0.0);
        static mut DIFF: (f64, f64) =  (0.0, 0.0);
        static mut PRESSED: bool =  false;
        static mut MODEL: (f32, f32) = (0.0, 0.0);
        static mut CAMERA: (f32, f32) = (0.0, 0.0);
        
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
                    
                    render_manager.render(size, unsafe {DIFF}, unsafe {ZOOM}, unsafe {MODEL}, unsafe {CAMERA});
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
            WindowEvent::MouseInput { device_id: _, state, button:_} => {
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
            WindowEvent::KeyboardInput {event, ..} => {
                if event.state == winit::event::ElementState::Pressed {
                    match event.key_without_modifiers().as_ref() {
                        Key::Character("w") => unsafe { MODEL.1 += 0.1; },
                        Key::Character("s") => unsafe { MODEL.1 -= 0.1; },
                        Key::Character("a") => unsafe { MODEL.0 -= 0.1; },
                        Key::Character("d") => unsafe { MODEL.0 += 0.1; },
                        Key::Named(winit::keyboard::NamedKey::ArrowUp) => unsafe { CAMERA.1 += 0.1; },
                        Key::Named(winit::keyboard::NamedKey::ArrowDown) => unsafe { CAMERA.1 -= 0.1; },
                        Key::Named(winit::keyboard::NamedKey::ArrowLeft) => unsafe { CAMERA.0 -= 0.1; },
                        Key::Named(winit::keyboard::NamedKey::ArrowRight) => unsafe { CAMERA.0 += 0.1; },
                        Key::Named(winit::keyboard::NamedKey::Escape) => {
                            event_loop.exit();
                        },
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }
}