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
use rayon::prelude::*;
use crate::utils;

pub struct RenderManager {
    gl: glow::Context,
    surface: Surface<WindowSurface>,
    context: glutin::context::PossiblyCurrentContext,
	shader_program: glow::Program,
	vao: glow::VertexArray,
	//start_time: std::time::Instant,
    num_indices: i32,
}

impl RenderManager {    
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

        let (vertices, indices)  = utils::load_mesh("objs/Guitar_01_OBJ/Guitar_01.obj");

        let texture_paths = vec![
            "objs/Guitar_01_OBJ/Guitar_01_Textures_Unity/guitar_01_AlbedoTransparency.png",
            "objs/Guitar_01_OBJ/Guitar_01_Textures_Unity/guitar_01_AO.png",
            "objs/Guitar_01_OBJ/Guitar_01_Textures_Unity/guitar_01_MetallicSmoothness.png",
            "objs/Guitar_01_OBJ/Guitar_01_Textures_Unity/guitar_01_Normal.png",
        ];


        let images: Vec<_> = utils::profile("images", || {texture_paths
            .par_iter()
            .map(|path| utils::get_image_data(path))
            .collect()});

        let textures: Vec<_> = images
            .into_iter()
            .map(|image| utils::generate_texture(&gl, image).unwrap())
            .collect();

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

            let vertex_source = utils::load_shader("shaders/modelvertexshader.glsl");
            let fragment_source = utils::load_shader("shaders/modelfragmentshader.glsl");
            // Create shader program first
            let vertex_shader = utils::compile_shader(&gl, &vertex_source, glow::VERTEX_SHADER);
            let fragment_shader = utils::compile_shader(&gl, &fragment_source, glow::FRAGMENT_SHADER);
            let shader_program = utils::create_shader_program(&gl, vertex_shader, fragment_shader);

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(textures[0]));
            let albedotexture_loc = gl.get_uniform_location(shader_program, "albedoMap");
            gl.uniform_1_i32(albedotexture_loc.as_ref(), 0);
            
            gl.active_texture(glow::TEXTURE1);
            gl.bind_texture(glow::TEXTURE_2D, Some(textures[1]));
            let ao_loc = gl.get_uniform_location(shader_program, "aoMap");
            gl.uniform_1_i32(ao_loc.as_ref(), 1);
            
            gl.active_texture(glow::TEXTURE2);
            gl.bind_texture(glow::TEXTURE_2D, Some(textures[2]));
            let metallic_loc = gl.get_uniform_location(shader_program, "metallicMap");
            gl.uniform_1_i32(metallic_loc.as_ref(), 2);
            
            gl.active_texture(glow::TEXTURE3);
            gl.bind_texture(glow::TEXTURE_2D, Some(textures[3]));
            let normal_loc = gl.get_uniform_location(shader_program, "normalMap");
            gl.uniform_1_i32(normal_loc.as_ref(), 3);

            // Create instance with actual shader program
            Self {
                gl,
                surface,
                context,
                shader_program,
                vao,
                //start_time: std::time::Instant::now(),
                num_indices: indices.len() as i32,
            }
		}
    }

    pub fn render(&self, size: (u32, u32), mouse: (f64, f64), scroll: f64, modelpos: (f32, f32), camera: (f32, f32)) {
       
        //let time = self.start_time.elapsed().as_secs_f32();
        
        let camera_pos = Vec3::new(0.0, 0.0, 5.0 * scroll as f32);

        let rotation = Mat4::from_rotation_y(mouse.0 as f32 * 0.005) * Mat4::from_rotation_x(mouse.1 as f32 * 0.005);
        let translation = Mat4::from_translation(Vec3::new(modelpos.0, modelpos.1, 0.0));
        let model_matrix = rotation * translation;
        
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