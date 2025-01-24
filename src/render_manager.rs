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
use std::{ffi::CString, fs};
use std::path::PathBuf;
use glam::{Vec3, Mat4};

pub struct RenderManager {
    gl: glow::Context,
    surface: Surface<WindowSurface>,
    context: glutin::context::PossiblyCurrentContext,
	shader_program: glow::Program,
	vao: glow::VertexArray,
	start_time: std::time::Instant,
    num_indices: i32,
}

impl RenderManager {

    fn get_asset_path(relative_path: &str) -> PathBuf {
        let base_dir = if cfg!(debug_assertions) {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        } else {
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf()
        };
        
        return base_dir.join(relative_path)
    }

    fn load_mesh(path: &str) -> (Vec<f32>, Vec<u32>) {
        let obj_path = Self::get_asset_path(path);

        let (models, _) = tobj::load_obj(&obj_path, &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        })
        .expect("Failed to load OBJ file");

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for model in models {
            let mesh = &model.mesh;
            
            // Store vertices
            for i in 0..mesh.positions.len() / 3 {
                // Position
                vertices.push(mesh.positions[i * 3]);
                vertices.push(mesh.positions[i * 3 + 1]);
                vertices.push(mesh.positions[i * 3 + 2]);
                
                // Normal
                if !mesh.normals.is_empty() {
                    vertices.push(mesh.normals[i * 3]);
                    vertices.push(mesh.normals[i * 3 + 1]);
                    vertices.push(mesh.normals[i * 3 + 2]);
                } else {
                    vertices.extend_from_slice(&[0.0, 0.0, 0.0]);
                }
                
                // UV
                if !mesh.texcoords.is_empty() {
                    vertices.push(mesh.texcoords[i * 2]);
                    vertices.push(mesh.texcoords[i * 2 + 1]);
                } else {
                    vertices.extend_from_slice(&[0.0, 0.0]);
                }
            }

            // Store indices
            indices.extend_from_slice(&mesh.indices);
        }

        (vertices, indices)
    }

    fn load_shader(shader_path: &str) -> String {
        let shader_path = Self::get_asset_path(shader_path);
        fs::read_to_string(&shader_path)
            .unwrap_or_else(|_| panic!("Failed to read shader file: {}", shader_path.display()))
    }
	fn compile_shader(gl: &glow::Context, source: &str, shader_type: u32) -> glow::Shader {
        unsafe {
            let shader = gl.create_shader(shader_type).expect("Cannot create shader");
            gl.shader_source(shader, source);
            gl.compile_shader(shader);

            if !gl.get_shader_compile_status(shader) {
                panic!("Failed to compile shader: {}", gl.get_shader_info_log(shader));
            }
            shader
        }
    }

    fn create_shader_program(gl: &glow::Context, vertex_shader: glow::Shader, fragment_shader: glow::Shader) -> glow::Program {
        unsafe {
            let program = gl.create_program().expect("Cannot create program");
            gl.attach_shader(program, vertex_shader);
            gl.attach_shader(program, fragment_shader);
            gl.link_program(program);

            if !gl.get_program_link_status(program) {
                panic!("Failed to link program: {}", gl.get_program_info_log(program));
            }

            gl.delete_shader(vertex_shader);
            gl.delete_shader(fragment_shader);
            program
        }
    }
    pub fn new(window: &Window) -> Self {
        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .build();

        // Create display
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

        let (vertices, indices)  = Self::load_mesh("objs/monkey.obj");


        unsafe {
            let vao = gl.create_vertex_array().unwrap();
            let vbo = gl.create_buffer().unwrap();
            let ebo = gl.create_buffer().unwrap();
            
            gl.bind_vertex_array(Some(vao));

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&vertices),
                glow::STATIC_DRAW,
            );

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&indices),
                glow::STATIC_DRAW,
            );

            const POSITION_ATTRIB: u32 = 0;
            const NORMAL_ATTRIB: u32 = 1;
            const TEXCOORD_ATTRIB: u32 = 2;

            const VERTEX_SIZE: usize = std::mem::size_of::<f32>();
            const STRIDE: i32 = (8 * VERTEX_SIZE) as i32; // 3 pos + 3 normal + 2 uv = 8 floats

            const POSITION_OFFSET: i32 = 0;
            const NORMAL_OFFSET: i32 = (3 * VERTEX_SIZE) as i32;
            const TEXCOORD_OFFSET: i32 = (6 * VERTEX_SIZE) as i32;

            gl.vertex_attrib_pointer_f32(
                POSITION_ATTRIB,
                3,  // vec3
                glow::FLOAT,
                false,
                STRIDE,
                POSITION_OFFSET
            );

            gl.vertex_attrib_pointer_f32(
                NORMAL_ATTRIB,
                3,  // vec3
                glow::FLOAT,
                false,
                STRIDE,
                NORMAL_OFFSET
            );

            gl.vertex_attrib_pointer_f32(
                TEXCOORD_ATTRIB,
                2,  // vec2
                glow::FLOAT,
                false,
                STRIDE,
                TEXCOORD_OFFSET
            );

            // Don't forget to enable all attribute arrays
            gl.enable_vertex_attrib_array(POSITION_ATTRIB);
            gl.enable_vertex_attrib_array(NORMAL_ATTRIB);
            gl.enable_vertex_attrib_array(TEXCOORD_ATTRIB);

            let vertex_source = Self::load_shader("shaders/modelvertexshader.glsl");
            let fragment_source = Self::load_shader("shaders/modelfragmentshader.glsl");
            // Create shader program first
            let vertex_shader = Self::compile_shader(&gl, &vertex_source, glow::VERTEX_SHADER);
            let fragment_shader = Self::compile_shader(&gl, &fragment_source, glow::FRAGMENT_SHADER);
            let shader_program = Self::create_shader_program(&gl, vertex_shader, fragment_shader);

            // Create instance with actual shader program
            Self {
                gl,
                surface,
                context,
                shader_program,
                vao,
                start_time: std::time::Instant::now(),
                num_indices: indices.len() as i32,
            }
		}
    }

    pub fn render(&self, size: (u32, u32), mouse: (f64, f64), scroll: f64) {
       
        let time = self.start_time.elapsed().as_secs_f32();
        
        let camera_pos = Vec3::new(0.0, 0.0, 5.0 * scroll as f32);
        let camera_direction = Vec3::new(0.0, 0.0, 0.0);

        let rotation = Mat4::from_rotation_y(mouse.0 as f32 * 0.005) * Mat4::from_rotation_x(mouse.1 as f32 * 0.005);
        let model_matrix = rotation;
        let view_matrix = Mat4::look_at_rh(
            camera_pos, // Camera position
            camera_direction, // Look at
            Vec3::new(0.0, 1.0, 0.0), // Up vector
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
                model_loc.as_ref(),
                false,
                &model_matrix.to_cols_array(),
            );
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

            let camera_loc = self.gl.get_uniform_location(self.shader_program, "lightPos");
            self.gl.uniform_3_f32(
                camera_loc.as_ref(),
                camera_pos.x,
                camera_pos.y,
                camera_pos.z,);
            self.gl.enable(glow::DEPTH_TEST);
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
           
            self.gl.viewport(0, 0, (size.0) as i32, (size.1) as i32);
            
            self.gl.use_program(Some(self.shader_program));

            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            
            self.gl.bind_vertex_array(Some(self.vao));
            self.gl.draw_elements(
                glow::TRIANGLES,
                self.num_indices,
                glow::UNSIGNED_INT,
                0,
            );
            
            self.surface.swap_buffers(&self.context).unwrap();
        }
    }
}