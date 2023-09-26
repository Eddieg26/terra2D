use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::transform::Transform;

use super::Sprite;

pub struct Color([f32; 3]);

impl Color {
    pub fn new() -> Color {
        Color([1.0; 3])
    }

    pub fn set_red(&mut self, value: f32) {
        self.0[0] = value;
    }

    pub fn set_green(&mut self, value: f32) {
        self.0[1] = value;
    }

    pub fn set_blue(&mut self, value: f32) {
        self.0[2] = value;
    }

    pub fn set(&mut self, red: f32, green: f32, blue: f32) {
        self.0[0] = red;
        self.0[1] = green;
        self.0[2] = blue;
    }

    pub fn get(&self) -> &[f32; 3] {
        &self.0
    }
}

pub struct SpriteRenderer {
    id: u64,
    transform: Transform,
    sprite: Sprite,
    color: Color,
}

impl SpriteRenderer {
    pub fn new(sprite: Sprite) -> SpriteRenderer {
        let mut hasher = DefaultHasher::new();
        uuid::Uuid::new_v4().to_string().hash(&mut hasher);

        SpriteRenderer {
            id: hasher.finish(),
            transform: Transform::new(),
            sprite,
            color: Color::new(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn sprite(&self) -> &Sprite {
        &self.sprite
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    pub fn color(&self) -> &Color {
        &self.color
    }

    pub fn color_mut(&mut self) -> &mut Color {
        &mut self.color
    }
}
