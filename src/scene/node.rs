use std::{any::Any, cell::RefCell, ops::Mul, rc::Rc};

use bytemuck::Zeroable;
use nalgebra::{Matrix4, Point3, Quaternion, Rotation3, UnitQuaternion, Vector2, Vector3};

use crate::{
    math::rect::Rect,
    renderer::surface::{Surface, SurfaceSharedData},
    utils::pool::Handle, resource::Resource,
};
#[derive(Debug)]
pub struct Light {
    radius: f32,
    color: Vector3<f32>,
}

impl Light {
    pub fn default() -> Light {
        Light {
            radius: 10.0,
            color: Vector3::new(1., 1., 1.),
        }
    }
}
#[derive(Debug)]
pub struct Camera {
    fov: f32,
    z_near: f32,
    z_far: f32,
    viewport: Rect<f32>,
    view_matrix: Matrix4<f32>,
    projection_matrix: Matrix4<f32>,
}

impl Camera {
    pub fn default() -> Camera {
        let fov: f32 = 45.0;
        let z_near: f32 = 1.;
        let z_far: f32 = 1000.;

        Camera {
            fov,
            z_near,
            z_far,
            view_matrix: Matrix4::identity(),
            projection_matrix: Matrix4::new_perspective(1.0, fov.to_radians(), z_near, z_far),
            viewport: Rect::<f32> {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        }
    }

    pub fn calculate_matrices(
        &mut self,
        pos: Point3<f32>,
        look: Point3<f32>,
        up: Vector3<f32>,
        aspect: f32,
    ) {
        let point = Point3::new(pos.x + look.x, pos.y + look.y, pos.z + look.z);

        self.view_matrix = Matrix4::look_at_rh(&pos, &point, &up);

        self.projection_matrix =
            Matrix4::new_perspective(aspect, self.fov.to_radians(), self.z_near, self.z_far);
    }

    pub fn get_viewport_pixels(&self, client_size: Vector2<f32>) -> Rect<i32> {
        Rect {
            x: (self.viewport.x * client_size.x) as i32,
            y: (self.viewport.y * client_size.y) as i32,
            width: (self.viewport.width * client_size.x) as i32,
            height: (self.viewport.height * client_size.y) as i32,
        }
    }

    pub fn get_view_projection_matrix(&self) -> Matrix4<f32> {
        self.projection_matrix * self.view_matrix
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub(crate) surfaces: Vec<Surface>,
}

impl Mesh {
    pub fn default() -> Mesh {
        Mesh {
            surfaces: Vec::new(),
        }
    }

    pub fn make_cube(&mut self) {
        self.surfaces.clear();
        let data = Rc::new(RefCell::new(SurfaceSharedData::make_cube()));
        self.surfaces.push(Surface::new(&data));
    }

    pub fn apply_texture(&mut self, tex: Rc<RefCell<Resource>>) {
        for surface in self.surfaces.iter_mut() {
            surface.set_texture(tex.clone());
        }
    }

    
}

#[derive(Debug)]
pub enum NodeKind {
    Base,
    Light(Light),
    Camera(Camera),
    Mesh(Mesh),

    /// User-defined node kind
    Custom(Box<dyn Any>),
}

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub kind: NodeKind,
    local_scale: Vector3<f32>,
    local_position: Vector3<f32>,
    local_rotation: UnitQuaternion<f32>,
    pre_rotation: UnitQuaternion<f32>,
    post_rotation: UnitQuaternion<f32>,
    rotation_offset: Vector3<f32>,
    rotation_pivot: Vector3<f32>,
    scaling_offset: Vector3<f32>,
    scaling_pivot: Vector3<f32>,
    pub(super) parent: Handle<Node>,
    pub(crate) children: Vec<Handle<Node>>,
    pub local_transform: Matrix4<f32>,
    pub(crate) global_transform: Matrix4<f32>,
}

impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Node {
            kind,
            name: String::from("Node"),
            children: Vec::new(),
            parent: Handle::none(),
            local_position: Vector3::zeros(),
            local_scale: Vector3::new(1., 1., 1.),
            local_rotation: UnitQuaternion::identity(),
            pre_rotation: UnitQuaternion::identity(),
            post_rotation: UnitQuaternion::identity(),
            rotation_offset: Vector3::zeros(),
            rotation_pivot: Vector3::zeros(),
            scaling_offset: Vector3::zeros(),
            scaling_pivot: Vector3::zeros(),
            local_transform: Matrix4::identity(),
            global_transform: Matrix4::identity(),
        }
    }

    pub fn calculate_local_transform(&mut self) {
        let pre_rotation = self.pre_rotation.to_homogeneous();
        let post_rotation = self.post_rotation.to_homogeneous().try_inverse().unwrap();
        let rotation = self.local_rotation.to_homogeneous();
        let scale = Matrix4::new_nonuniform_scaling(&self.local_scale);

        let translation = Matrix4::new_translation(&self.local_position);

        let rotation_offset = Matrix4::new_rotation(self.rotation_offset);
        let rotation_pivot = Matrix4::new_rotation(self.rotation_pivot);
        let rotation_pivot_inv = rotation_pivot.try_inverse().unwrap();
        let scale_offset = Matrix4::from_scaled_axis(self.scaling_offset);
        let scale_pivot = Matrix4::from_scaled_axis(self.scaling_pivot);
        let scale_pivot_inv = scale_pivot.try_inverse().unwrap();

        self.local_transform = translation
            * rotation_offset
            * rotation_pivot
            * pre_rotation
            * rotation
            * post_rotation
            * rotation_pivot_inv
            * scale_offset
            * scale_pivot
            * scale
            * scale_pivot_inv;
    }

    pub fn borrow_kind(&self) -> &NodeKind {
        &self.kind
    }

    pub fn borrow_kind_mut(&mut self) -> &mut NodeKind {
        &mut self.kind
    }

    pub fn set_local_position(&mut self, pos: Vector3<f32>) {
        self.local_position = pos;
    }

    pub fn set_local_rotation(&mut self, rot: UnitQuaternion<f32>) {
        self.local_rotation = rot;
    }

    pub fn set_local_scale(&mut self, scl: Vector3<f32>) {
        self.local_scale = scl;
    }

    pub fn offset(&mut self, vec: Vector3<f32>) {
        self.local_position += &vec;
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn get_global_position(&self) -> Vector3<f32> {
        Vector3::new(
            self.global_transform[12],
            self.global_transform[13],
            self.global_transform[14],
        )
    }

    pub fn get_look_vector(&self) -> Vector3<f32> {
        Vector3::new(
            self.global_transform[8],
            self.global_transform[9],
            self.global_transform[10],
        )
    }

    pub fn get_side_vector(&self) -> Vector3<f32> {
        Vector3::new(
            self.global_transform[0],
            self.global_transform[1],
            self.global_transform[2],
        )
    }

    pub fn get_up_vector(&self) -> Vector3<f32> {
        Vector3::new(
            self.global_transform[4],
            self.global_transform[5],
            self.global_transform[6],
        )
    }
}
