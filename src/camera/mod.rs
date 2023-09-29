use nalgebra_glm as glm;

use crate::{
    terra::data::{ClippingPlanes, Viewport},
    transform::Transform,
};

pub struct Camera {
    transform: Transform,
    size: f32,
    clipping_planes: ClippingPlanes,
    viewport: Viewport,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            size: 5.0,
            transform: Transform::new(),
            clipping_planes: ClippingPlanes::default(),
            viewport: Viewport::default(),
        }
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn clipping_planes(&self) -> &ClippingPlanes {
        &self.clipping_planes
    }

    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size
    }

    pub fn set_clipping_planes(&mut self, clipping_planes: ClippingPlanes) {
        self.clipping_planes = clipping_planes;
    }

    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport
    }

    pub fn ortho(&self) -> glm::Mat4 {
        let clipping = self.clipping_planes();
        let size = self.size();

        glm::ortho(-size, size, -size, size, clipping.near, clipping.far)
    }
}
