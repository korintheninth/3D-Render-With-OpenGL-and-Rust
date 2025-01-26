use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext},
    display::{Display, DisplayApiPreference},
    prelude::*,
    surface::{Surface, WindowSurface},
};
use glow::HasContext;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;
use std::ffi::CString;
use glam::{Vec3, Mat4};
use crate::utils;

pub struct RenderManager {
    gl: glow::Context,
    surface: Surface<WindowSurface>,
    context: glutin::context::PossiblyCurrentContext,
	shader_program: glow::Program,
	vaos: Vec<glow::VertexArray>,
	//start_time: std::time::Instant,
    nums_of_indices: Vec<i32>,
}

impl RenderManager {    
    pub fn new(window: &Window) -> Self {
        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .build();

        let display = unsafe {
            Display::new(
                window.display_handle()
                    .map_err(|e| e.to_string())
                    .unwrap()
                    .as_raw(),
                DisplayApiPreference::Wgl(None)
            )
            .expect("Failed to create display")
        };

        let config = unsafe {
            display
                .find_configs(template)
                .expect("Failed to find configs")
                .next()
                .expect("No config found")
        };

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(None))
            .build(Some(
                window.window_handle()
                    .map_err(|e| e.to_string())
                    .unwrap()
                    .as_raw()
            ));

        let size = window.inner_size();
        let surface_attributes = 
            glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new().build(
                window.window_handle()
                    .map_err(|e| e.to_string())
                    .unwrap()
                    .as_raw(),
                std::num::NonZeroU32::new(size.width).unwrap(),
                std::num::NonZeroU32::new(size.height).unwrap(),
            );

        let surface = unsafe {
            display
                .create_window_surface(&config, &surface_attributes)
                .expect("Failed to create surface")
        };

        let context = unsafe {
            display
                .create_context(&config, &context_attributes)
                .expect("Failed to create context")
                .make_current(&surface)
                .expect("Failed to make context current")
        };

        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                let s = CString::new(s).unwrap();
                display.get_proc_address(s.as_c_str()) as *const _
            })
        };
        
        let vertex_source = utils::load_shader("shaders/modelvertexshader.glsl");
        let fragment_source = utils::load_shader("shaders/modelfragmentshader.glsl");
        // Create shader program first
        let vertex_shader = utils::compile_shader(&gl, &vertex_source, glow::VERTEX_SHADER);
        let fragment_shader = utils::compile_shader(&gl, &fragment_source, glow::FRAGMENT_SHADER);
        let shader_program = utils::create_shader_program(&gl, vertex_shader, fragment_shader);
        
        let mut vaos: Vec<glow::VertexArray>;
        let mut nums_of_indices: Vec<i32>;

        let (mut vao, mut num_indices) = utils::load_model_with_textures(&gl, &shader_program, "objs/Guitar_01_OBJ/Guitar_01.obj");
        vaos = vec![vao];
        nums_of_indices = vec![num_indices];
        (vao, num_indices) = utils::load_model_with_textures(&gl, &shader_program, "objs/Guitar_01_OBJ/Guitar_01.obj");
        vaos.push(vao);
        nums_of_indices.push(num_indices);

        Self {
            gl,
            surface,
            context,
            shader_program,
            vaos,
            //start_time: std::time::Instant::now(),
            nums_of_indices,
		}
    }

    pub fn render(&self, size: (u32, u32), mouse: (f64, f64), scroll: f64, modelpos: (f32, f32), camera: (f32, f32)) {
       
        //let time = self.start_time.elapsed().as_secs_f32();
        
        let camera_pos = Vec3::new(0.0, 0.0, 5.0 * scroll as f32);
        
        let y_rot = camera.0 as f32 * 0.5;
        let x_rot = camera.1 as f32 * 0.5;

        let camera_direction = Vec3::new(
            y_rot.sin() * x_rot.cos(),
            -x_rot.sin(),
            y_rot.cos() * x_rot.cos()
        ).normalize();

        let view_matrix = Mat4::look_at_rh(
            camera_pos,
            camera_pos - camera_direction,
            Vec3::new(0.0, 1.0, 0.0),
        );

        //let projection_matrix = Mat4::orthographic_lh(-5.0, 5.0, -5.0, 5.0, 0.1, 100.0);
        let projection_matrix = Mat4::perspective_rh(
            45.0_f32.to_radians(),
            size.0 as f32 / size.1 as f32,
            0.1,
            100.0,
        );
        unsafe {
            
            let model_loc = self.gl.get_uniform_location(self.shader_program, "model");
            let view_loc = self.gl.get_uniform_location(self.shader_program, "view");
            let proj_loc = self.gl.get_uniform_location(self.shader_program, "projection");

            self.gl.uniform_matrix_4_f32_slice(
                view_loc.as_ref(),
                false,
                &view_matrix.to_cols_array(),
            );
            self.gl.uniform_matrix_4_f32_slice(
                proj_loc.as_ref(),
                false,
                &projection_matrix.to_cols_array(),
            );

            let camera_loc = self.gl.get_uniform_location(self.shader_program, "cameraPos");
            self.gl.uniform_3_f32(
                camera_loc.as_ref(),
                camera_pos.x,
                camera_pos.y,
                camera_pos.z,);
            self.gl.enable(glow::DEPTH_TEST);
            let camera_dir_loc = self.gl.get_uniform_location(self.shader_program, "cameraDir");
            self.gl.uniform_3_f32(
                camera_dir_loc.as_ref(),
                camera_direction.x,
                camera_direction.y,
                camera_direction.z,);

            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
           
            self.gl.viewport(0, 0, (size.0) as i32, (size.1) as i32);
            
            self.gl.use_program(Some(self.shader_program));

            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            
            for i in 0..self.vaos.len() {

                let rotation = Mat4::from_rotation_y(mouse.0 as f32 * 0.005) * Mat4::from_rotation_x(mouse.1 as f32 * 0.005);
                let translation = Mat4::from_translation(Vec3::new(modelpos.0 + 2.5  - 5.0 * i as f32, modelpos.1, 0.0));
                let model_matrix = rotation * translation;

                self.gl.uniform_matrix_4_f32_slice(
                    model_loc.as_ref(),
                    false,
                    &model_matrix.to_cols_array(),
                );
                
                self.gl.bind_vertex_array(Some(self.vaos[i]));
                self.gl.draw_elements(
                    glow::TRIANGLES,
                    self.nums_of_indices[i],
                    glow::UNSIGNED_INT,
                    0,
                );
            }
            
            self.surface.swap_buffers(&self.context).unwrap();
        }
    }
}