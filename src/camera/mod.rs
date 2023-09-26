use crate::transform::Transform;

#[derive(Clone, Copy)]
pub struct ClippingPlanes {
    pub near: f32,
    pub far: f32,
}

pub struct Camera {
    transform: Transform,
    size: f32,
    clipping_planes: ClippingPlanes,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            transform: Transform::new(),
            size: 5.0,
            clipping_planes: ClippingPlanes {
                near: -1.0,
                far: 1.0,
            },
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

    pub fn set_size(&mut self, size: f32) {
        self.size = size
    }

    pub fn set_clipping_planes(&mut self, clipping_planes: ClippingPlanes) {
        self.clipping_planes = clipping_planes;
    }
}
