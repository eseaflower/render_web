use cgmath::prelude::*;
use std::mem;
use crate::view_state::{Zoom, ViewState};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}
unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    pub fn to_desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    format: wgpu::VertexFormat::Float3,
                    shader_location: 0,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float2,
                    shader_location: 1,
                },
            ],
        }
    }
}

// Vertex position in image space!
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [0.0, 1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
];

const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

pub struct Quad {
    vertices: Vec<Vertex>,
    indexes: Vec<u16>,
    shader_to_screen: ViewTransform,
    viewport_size: (f32, f32),
    texture_size: (f32, f32),
    image_size: (f32, f32),
}

impl Quad {
    pub fn new() -> Self {
        Quad {
            vertices: VERTICES.iter().map(Vertex::clone).collect(),
            indexes: INDICES.iter().map(u16::clone).collect(),
            shader_to_screen: ViewTransform::identity(),
            viewport_size: (1_f32, 1_f32),
            texture_size: (1_f32, 1_f32),
            image_size: (1_f32, 1_f32),
        }
    }

    pub fn with_init(viewport_size: (f32, f32)) -> Self {
        let mut quad = Quad::default();
        quad.set_viewport_size(viewport_size);
        quad
    }

    fn compute_image_to_screen(&self, state:&ViewState) -> ViewTransform {

        let mut transform = match state.zoom {
            Zoom::Fit(mag) => {
                let x_scale = self.viewport_size.0 / self.image_size.0;
                let y_scale = self.viewport_size.1 / self.image_size.1;
                let scale = x_scale.min(y_scale) * mag;
                ViewTransform::scale_diag(scale)
            },
            Zoom::Pixel(mag) => {
                ViewTransform::scale_diag(mag)
            },
        };

        // Always center the image after zoom
        let xform_center = transform.transform_vertex(&[
            self.image_size.0 / 2.0,
            self.image_size.1 / 2.0,
            1.0,
        ]);
        let vp_center = (self.viewport_size.0 / 2.0, self.viewport_size.1 / 2.0);
        let mut x_trans = vp_center.0 - xform_center[0];
        let mut y_trans = vp_center.1 - xform_center[1];
        // Add displacement
        let disp = state.get_displacement();
        x_trans += disp.0;
        y_trans += disp.1;

        transform
            .compose_mut(&ViewTransform::translate(x_trans, y_trans));

        transform
    }

    pub fn get_vertex(&self, state: &ViewState) -> Vec<Vertex> {
        //&self.vertices
        let mut vertex_tranform = self.compute_image_to_screen(state);//self.image_to_screen.clone();

        vertex_tranform.compose_mut(&self.shader_to_screen.invert());

        let vspace = vertex_tranform.transform_vertex(&[1_f32,1_f32,1_f32]);
        let sspace = vertex_tranform.invert().transform_vertex(&vspace);
        //log::info!("Transformed: {}, {}", vspace[0], vspace[1]);


        let v: Vec<_> = self
            .vertices
            .iter()
            .map(|x| Vertex {
                position: vertex_tranform.transform_vertex(&x.position),
                tex_coords: x.tex_coords,
            })
            .collect();
        //        dbg!(&v);
        v
    }
    pub fn index_ref(&self) -> &[u16] {
        &self.indexes
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertices.len() as u32
    }

    pub fn index_count(&self) -> u32 {
        self.indexes.len() as u32
    }

    pub fn map_texture_coords(&mut self, img_dims: (f32, f32), tex_dims: (f32, f32)) {
        let u = img_dims.0 / tex_dims.0;
        let v = img_dims.1 / tex_dims.1;
        self.vertices
            .iter_mut()
            .zip(VERTICES.iter())
            .for_each(|(v1, v2)| {
                v1.position = [
                    v2.position[0] * img_dims.0,
                    v2.position[1] * img_dims.1,
                    v1.position[2],
                ];
                v1.tex_coords = [v2.tex_coords[0] * u, v2.tex_coords[1] * v];
            });
        self.image_size = img_dims;
        self.texture_size = tex_dims;
    }

    pub fn set_viewport_size(&mut self, size: (f32, f32)) {
        // Update the shader to screen transform.
        // Swap y-axis direction and normalize to unit square
        self.shader_to_screen = ViewTransform::scale(0.5, -0.5);
        // Translate
        self.shader_to_screen
            .compose_mut(&ViewTransform::translate(0.5, 0.5));
        // Scale to viewport size
        self.shader_to_screen
            .compose_mut(&ViewTransform::scale(size.0, size.1));

        self.viewport_size = size;
    }

    // pub fn zoom_fit(&mut self) {
    //     let x_scale = self.viewport_size.0 / self.image_size.0;
    //     let y_scale = self.viewport_size.1 / self.image_size.1;
    //     let scale = x_scale.min(y_scale);
    //     self.image_to_screen = ViewTransform::scale_diag(scale);
    //     let xform_center = self.image_to_screen.transform_vertex(&[
    //         self.image_size.0 / 2.0,
    //         self.image_size.1 / 2.0,
    //         1.0,
    //     ]);
    //     let x_trans = self.viewport_size.0 / 2.0 - xform_center[0];
    //     let y_trans = self.viewport_size.1 / 2.0 - xform_center[1];
    //     self.image_to_screen
    //         .compose_mut(&ViewTransform::translate(x_trans, y_trans));
    // }
}

impl Default for Quad {
    fn default() -> Self {
        Quad::new()
    }
}

#[derive(Debug)]
pub struct ViewTransform {
    mat: cgmath::Matrix3<f32>,
}

impl ViewTransform {
    fn unit_mat() -> cgmath::Matrix3<f32> {
        cgmath::Matrix3::from_angle_z(cgmath::Rad(0.0))
    }

    pub fn identity() -> Self {
        ViewTransform {
            mat: ViewTransform::unit_mat(),
        }
    }

    pub fn scale(x: f32, y: f32) -> Self {
        let mut mat = ViewTransform::unit_mat();
        mat.x.x = x;
        mat.y.y = y;
        ViewTransform { mat }
    }

    pub fn scale_diag(s: f32) -> Self {
        ViewTransform::scale(s, s)
    }

    pub fn translate(x: f32, y: f32) -> Self {
        let mut mat = ViewTransform::unit_mat();
        mat.z.x = x;
        mat.z.y = y;
        ViewTransform { mat }
    }

    pub fn compose(&self, other: &ViewTransform) -> Self {
        let mat = other.mat * self.mat;
        ViewTransform { mat }
    }

    pub fn compose_mut(&mut self, other: &ViewTransform) {
        *self = self.compose(other);
    }

    pub fn invert(&self) -> ViewTransform {
        let mat = self.mat.invert().unwrap();
        ViewTransform { mat }
    }

    pub fn transform_vertex(&self, v: &[f32; 3]) -> [f32; 3] {
        // Set z = 1.0 to be affected by translations
        let mut v = cgmath::vec3(v[0], v[1], 1.0);
        v = self.mat * v;
        // return the transformed vertex
        [v.x, v.y, 0.0]
    }
}

impl Clone for ViewTransform {
    fn clone(&self) -> Self {
        ViewTransform { mat: self.mat }
    }
}



#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test() {
        let mut state = ViewState::new();
        state.set_anchor((256.0, 256.0));
        state.set_position((0.0, 0.0));
        let mut q = Quad::new();
        q.set_viewport_size((512_f32, 512_f32));
        q.map_texture_coords((512_f32, 512_f32), (1024_f32, 1024_f32));
        let v = q.get_vertex(&state);
        dbg!(v);
    }
}
