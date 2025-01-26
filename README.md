# 3D rendering with OpenGl in rust
Simple 3D renderer using OpenGL, implemented in Rust.

## Features
- .obj file support
- PBR materials

## Requirements
- Rust
- OpenGL compatible GPU

## .obj and Texture File order
- Texture files should have the names: AlbedoTransparency.png, AO.png, MetallicSmoothness.png, Normal.png
- Folder Containing the textures should be in the same directory as the .obj file. For a .obj file with name modelName.obj, folder for textures should have the name modelNameTextures