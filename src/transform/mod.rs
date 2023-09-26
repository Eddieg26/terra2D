use glm::Mat4;
use nalgebra_glm::{self as glm, Vec2};

pub struct Transform {
    position: Vec2,
    scale: Vec2,
    rotation: f32,
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            position: Vec2::zeros(),
            scale: Vec2::new(1.0, 1.0),
            rotation: 0.0,
        }
    }

    pub fn position(&self) -> &Vec2 {
        &self.position
    }

    pub fn scale(&self) -> &Vec2 {
        &self.scale
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn matrix(&self) -> Mat4 {
        let mut model = glm::Mat4::identity();

        model = glm::scale(&model, &glm::vec3(self.scale.x, self.scale.y, 1.0));
        model = glm::rotate(
            &model,
            f32::to_radians(self.rotation),
            &glm::vec3(0.0, 0.0, 1.0),
        );
        model = glm::translate(&model, &glm::vec3(self.position.x, self.position.y, 0.0));

        model.into()
    }

    pub fn translate(&mut self, translation: &Vec2) -> &Vec2 {
        let new_position = self.position + translation;
        self.position = new_position;

        &self.position
    }

    pub fn rotate(&mut self, rotation: f32) -> f32 {
        self.rotation = self.rotation + rotation;
        self.rotation
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    pub fn set_scale(&mut self, scale: Vec2) {
        self.scale = scale;
    }
}
