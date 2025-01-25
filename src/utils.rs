use glow::HasContext;
use std::fs;
use std::path::PathBuf;

pub fn generate_texture(gl: &glow::Context, path: &str) -> Result<glow::Texture, Box<dyn std::error::Error>> {
    unsafe {
        // Load image
        let image = image::open(get_asset_path(path))?.flipv().into_rgba8();
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