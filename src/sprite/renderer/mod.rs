use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::terra::data::Color;
use crate::transform::Transform;

use super::Sprite;

pub struct SpriteRenderer {
    id: u64,
    name: Box<str>,
    transform: Transform,
    sprite: Sprite,
    color: Color,
}

impl SpriteRenderer {
    pub fn new(sprite: Sprite, name: &str) -> SpriteRenderer {
        let mut hasher = DefaultHasher::new();
        uuid::Uuid::new_v4().to_string().hash(&mut hasher);

        SpriteRenderer {
            id: hasher.finish(),
            transform: Transform::new(),
            name: name.into(),
            sprite,
            color: Color::new(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &Box<str> {
        &self.name
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
