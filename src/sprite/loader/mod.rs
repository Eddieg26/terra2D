use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use super::Sprite;

pub struct SpriteLoader {
    sprites: HashMap<u64, Sprite>,
}

impl SpriteLoader {
    pub fn get(&self, id: &u64) -> Option<Sprite> {
        if let Some(sprite) = self.sprites.get(id) {
            Some(sprite.clone())
        } else {
            None
        }
    }

    pub fn load() -> SpriteLoader {
        let mut sprites: HashMap<u64, Sprite> = HashMap::new();

        fs::read_dir("./src/assets/sprites")
            .unwrap()
            .for_each(|entry| {
                let path = entry.unwrap().path();
                let ext = path.extension().unwrap().to_str().unwrap();
                match ext {
                    "png" => Self::add_sprite(&mut sprites, &path),
                    "jpeg" => Self::add_sprite(&mut sprites, &path),
                    "jpg" => Self::add_sprite(&mut sprites, &path),
                    "tiff" => Self::add_sprite(&mut sprites, &path),
                    "bmp" => Self::add_sprite(&mut sprites, &path),
                    "tga" => Self::add_sprite(&mut sprites, &path),
                    _ => (),
                }
            });

        SpriteLoader { sprites }
    }

    fn add_sprite(sprites: &mut HashMap<u64, Sprite>, path: &Path) {
        let mut filepath = path.parent().unwrap().to_str().unwrap().to_owned();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        filepath.push_str("/");
        filepath.push_str(path.file_name().unwrap().to_str().unwrap());

        let mut full_path = std::env::current_dir().unwrap();
        full_path.push(PathBuf::from("src\\assets\\sprites\\"));
        full_path.push(file_name);

        let mut hasher = DefaultHasher::new();
        filepath.hash(&mut hasher);
        let id = hasher.finish();
        println!("FILEPATH: {}", filepath);

        let sprite = Sprite::new(
            id,
            full_path
                .to_str()
                .unwrap()
                .to_owned()
                .replace("\\", "/")
                .into(),
        );
        sprites.insert(id, sprite);
    }
}
