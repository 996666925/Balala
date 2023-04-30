use std::{cell::RefCell, mem::size_of, rc::Rc};

use glow::{HasContext, NativeBuffer, NativeVertexArray};
use nalgebra::{Vector2, Vector3, Vector4};

use crate::resource::{Resource, ResourceKind};

use super::renderer::GL;

#[derive(Debug)]
pub struct SurfaceSharedData {
    need_upload: bool,
    vbo: NativeBuffer,
    vao: NativeVertexArray,
    ebo: NativeBuffer,
    positions: Vec<Vector3<f32>>,
    normals: Vec<Vector3<f32>>,
    tex_coords: Vec<Vector2<f32>>,
    tangents: Vec<Vector4<f32>>,
    indices: Vec<i32>,
}

impl SurfaceSharedData {
    fn new() -> Self {
        unsafe {
            let gl = GL.get().unwrap();
            let mut vbo = gl.create_buffer().unwrap();
            let mut ebo = gl.create_buffer().unwrap();
            let mut vao = gl.create_vertex_array().unwrap();

            Self {
                need_upload: true,
                vbo,
                vao,
                ebo,
                positions: Vec::new(),
                normals: Vec::new(),
                tex_coords: Vec::new(),
                tangents: Vec::new(),
                indices: Vec::new(),
            }
        }
    }

    pub fn upload(&mut self) {
        unsafe {
            let gl = GL.get().unwrap();

            let positions_bytes = self.positions.len() * size_of::<Vector3<f32>>();
            let tex_coords_bytes = self.tex_coords.len() * size_of::<Vector2<f32>>();
            let normals_bytes = self.normals.len() * size_of::<Vector3<f32>>();
            let tangents_bytes = self.tangents.len() * size_of::<Vector4<f32>>();

            let total_size_bytes =
                positions_bytes + normals_bytes + tex_coords_bytes + tangents_bytes;

            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&self.indices),
                glow::STATIC_DRAW,
            );
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                total_size_bytes as i32,
                glow::STATIC_DRAW,
            );

            let pos_offset = 0usize;
            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                pos_offset as i32,
                bytemuck::cast_slice(&self.positions),
            );

            let tex_coord_offset = pos_offset + positions_bytes;

            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                tex_coord_offset as i32,
                bytemuck::cast_slice(&self.tex_coords),
            );

            let normals_offset = tex_coord_offset + tex_coords_bytes;

            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                normals_offset as i32,
                bytemuck::cast_slice(&self.normals),
            );

            let tangents_offset = normals_offset + normals_bytes;

            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                tangents_offset as i32,
                bytemuck::cast_slice(&self.tangents),
            );

            gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                size_of::<Vector3<f32>>() as i32,
                pos_offset as i32,
            );
            gl.enable_vertex_attrib_array(0);

            gl.vertex_attrib_pointer_f32(
                1,
                2,
                glow::FLOAT,
                false,
                size_of::<Vector2<f32>>() as i32,
                tex_coord_offset as i32,
            );
            gl.enable_vertex_attrib_array(1);

            gl.vertex_attrib_pointer_f32(
                2,
                3,
                glow::FLOAT,
                false,
                size_of::<Vector3<f32>>() as i32,
                normals_offset as i32,
            );
            gl.enable_vertex_attrib_array(2);

            gl.vertex_attrib_pointer_f32(
                3,
                4,
                glow::FLOAT,
                false,
                size_of::<Vector4<f32>>() as i32,
                tangents_offset as i32,
            );
            gl.enable_vertex_attrib_array(3);

            gl.bind_vertex_array(None);

            self.need_upload = false;
        }
    }

    pub fn calculate_tangents(&self) {}

    pub fn make_cube() -> Self {
        let mut data = Self::new();
        data.positions = vec![
            // Front
            Vector3::new(-0.5, -0.5, 0.5),
            Vector3::new(-0.5, 0.5, 0.5),
            Vector3::new(0.5, 0.5, 0.5),
            Vector3::new(0.5, -0.5, 0.5),
            // Back
            Vector3::new(-0.5, -0.5, -0.5),
            Vector3::new(-0.5, 0.5, -0.5),
            Vector3::new(0.5, 0.5, -0.5),
            Vector3::new(0.5, -0.5, -0.5),
            // Left
            Vector3::new(-0.5, -0.5, -0.5),
            Vector3::new(-0.5, 0.5, -0.5),
            Vector3::new(-0.5, 0.5, 0.5),
            Vector3::new(-0.5, -0.5, 0.5),
            // Right
            Vector3::new(0.5, -0.5, -0.5),
            Vector3::new(0.5, 0.5, -0.5),
            Vector3::new(0.5, 0.5, 0.5),
            Vector3::new(0.5, -0.5, 0.5),
            // Top
            Vector3::new(-0.5, 0.5, 0.5),
            Vector3::new(-0.5, 0.5, -0.5),
            Vector3::new(0.5, 0.5, -0.5),
            Vector3::new(0.5, 0.5, 0.5),
            // Bottom
            Vector3::new(-0.5, -0.5, 0.5),
            Vector3::new(-0.5, -0.5, -0.5),
            Vector3::new(0.5, -0.5, -0.5),
            Vector3::new(0.5, -0.5, 0.5),
        ];

        data.normals = vec![
            // Front
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 0.0, 1.0),
            // Back
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, 0.0, -1.0),
            // Left
            Vector3::new(-1.0, 0.0, 0.0),
            Vector3::new(-1.0, 0.0, 0.0),
            Vector3::new(-1.0, 0.0, 0.0),
            Vector3::new(-1.0, 0.0, 0.0),
            // Right
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            // Top
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            // Bottom
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
        ];

        data.tex_coords = vec![
            // Front
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 1.0),
            Vector2::new(1.0, 1.0),
            Vector2::new(1.0, 0.0),
            // Back
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 1.0),
            Vector2::new(1.0, 1.0),
            Vector2::new(1.0, 0.0),
            // Left
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 1.0),
            Vector2::new(1.0, 1.0),
            Vector2::new(1.0, 0.0),
            // Right
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 1.0),
            Vector2::new(1.0, 1.0),
            Vector2::new(1.0, 0.0),
            // Top
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 1.0),
            Vector2::new(1.0, 1.0),
            Vector2::new(1.0, 0.0),
            // Bottom
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 1.0),
            Vector2::new(1.0, 1.0),
            Vector2::new(1.0, 0.0),
        ];
        data.indices = vec![
            2, 1, 0, 3, 2, 0, 4, 5, 6, 4, 6, 7, 10, 9, 8, 11, 10, 8, 12, 13, 14, 12, 14, 15, 18,
            17, 16, 19, 18, 16, 20, 21, 22, 20, 22, 23,
        ];

        data
    }
}

impl Drop for SurfaceSharedData {
    fn drop(&mut self) {
        unsafe {
            let gl = GL.get().unwrap();
            gl.delete_buffer(self.vbo);
            gl.delete_buffer(self.ebo);
            gl.delete_vertex_array(self.vao);
        }
    }
}

type SurfaceSharedDataRef = Rc<RefCell<SurfaceSharedData>>;

#[derive(Debug)]
pub struct Surface {
    pub(crate) data: SurfaceSharedDataRef,
    pub(crate) texture: Option<Rc<RefCell<Resource>>>,
}

impl Surface {
    pub fn new(data: &SurfaceSharedDataRef) -> Self {
        Self {
            data: data.clone(),
            texture: None,
        }
    }
    pub fn set_texture(&mut self, tex: Rc<RefCell<Resource>>) {
        if let ResourceKind::Texture(_) = tex.borrow_mut().borrow_kind() {
            self.texture = Some(tex.clone());
        } else {
            self.texture = None;
        }
    }

    pub fn draw(&self) {
        unsafe {
            let gl = GL.get().unwrap();

            let mut data = self.data.borrow_mut();
            if data.need_upload {
                data.upload();
            }
            if let Some(ref resource) = self.texture {
                if let ResourceKind::Texture(texture) = &resource.borrow_mut().borrow_kind() {
                    gl.bind_texture(glow::TEXTURE_2D, texture.gpu_tex);
                }
            } else {
                gl.bind_texture(glow::TEXTURE_2D, None);
            }
            gl.bind_vertex_array(Some(data.vao));
            gl.draw_elements(
                glow::TRIANGLES,
                data.indices.len() as i32,
                glow::UNSIGNED_INT,
                0,
            );
        }
    }
}
