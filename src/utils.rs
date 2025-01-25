use glow::{HasContext, NativeProgram};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use rayon::prelude::*;

pub fn profile<F, T>(name: &str, f: F) -> T 
where 
    F: FnOnce() -> T 
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    println!("{} took {:?}", name, duration);
    result
}


pub fn load_model_with_textures(gl: &glow::Context, shader_program: &NativeProgram ,path: &str) -> (glow::VertexArray, i32) {
        let (vertices, indices)  = load_mesh(path);

        let albedo_path = path.strip_suffix(".obj").unwrap().to_owned() + "Textures/AlbedoTransparency.png";
		let ao_path = path.strip_suffix(".obj").unwrap().to_owned() + "Textures/AO.png";
		let metallic_path = path.strip_suffix(".obj").unwrap().to_owned() + "Textures/MetallicSmoothness.png";
		let normal_path = path.strip_suffix(".obj").unwrap().to_owned() + "Textures/Normal.png";
        let texture_paths = vec![
            albedo_path.as_str(),
			ao_path.as_str(),
			metallic_path.as_str(),
			normal_path.as_str()
        ];


        let images: Vec<_> = profile("images", || {texture_paths
            .par_iter()
            .map(|path| get_image_data(path))
            .collect()});

        let textures: Vec<_> = images
            .into_iter()
            .map(|image| generate_texture(&gl, image).unwrap())
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

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(textures[0]));
            let albedotexture_loc = gl.get_uniform_location(*shader_program, "albedoMap");
            gl.uniform_1_i32(albedotexture_loc.as_ref(), 0);
            
            gl.active_texture(glow::TEXTURE1);
            gl.bind_texture(glow::TEXTURE_2D, Some(textures[1]));
            let ao_loc = gl.get_uniform_location(*shader_program, "aoMap");
            gl.uniform_1_i32(ao_loc.as_ref(), 1);
            
            gl.active_texture(glow::TEXTURE2);
            gl.bind_texture(glow::TEXTURE_2D, Some(textures[2]));
            let metallic_loc = gl.get_uniform_location(*shader_program, "metallicMap");
            gl.uniform_1_i32(metallic_loc.as_ref(), 2);
            
            gl.active_texture(glow::TEXTURE3);
            gl.bind_texture(glow::TEXTURE_2D, Some(textures[3]));
            let normal_loc = gl.get_uniform_location(*shader_program, "normalMap");
            gl.uniform_1_i32(normal_loc.as_ref(), 3);
			
			(vao, indices.len() as i32)
		}
}

pub fn get_image_data(path: &str) -> image::RgbaImage {
		let image =  image::open(get_asset_path(path)).unwrap().flipv().into_rgba8();
		image
}

pub fn generate_texture(gl: &glow::Context, image: image::RgbaImage) -> Result<glow::Texture, Box<dyn std::error::Error>> {
    unsafe {
        // Load image
        let (width, height) = image.dimensions();

        // Create texture
        let texture = gl.create_texture()?;
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));

        // Set texture parameters
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR_MIPMAP_LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            glow::REPEAT as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            glow::REPEAT as i32,
        );

        // Upload texture data
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            width as i32,
            height as i32,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(Some(&image)),
        );

        // Generate mipmaps
        gl.generate_mipmap(glow::TEXTURE_2D);

        Ok(texture)
    }
}

pub fn get_asset_path(relative_path: &str) -> PathBuf {
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

pub fn load_mesh(path: &str) -> (Vec<f32>, Vec<u32>) {
    let obj_path = get_asset_path(path);

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

        indices.extend_from_slice(&mesh.indices);
    }

    (vertices, indices)
}

pub fn load_shader(shader_path: &str) -> String {
    let shader_path = get_asset_path(shader_path);
    fs::read_to_string(&shader_path)
        .unwrap_or_else(|_| panic!("Failed to read shader file: {}", shader_path.display()))
}

pub fn compile_shader(gl: &glow::Context, source: &str, shader_type: u32) -> glow::Shader {
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

pub fn create_shader_program(gl: &glow::Context, vertex_shader: glow::Shader, fragment_shader: glow::Shader) -> glow::Program {
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