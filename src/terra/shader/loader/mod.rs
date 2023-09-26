use std::{collections::HashMap, sync::Arc};

use vulkano::{device::Device, shader::ShaderModule};

pub struct ShaderLoader {
    vertex_shaders: HashMap<String, Arc<ShaderModule>>,
    fragment_shaders: HashMap<String, Arc<ShaderModule>>,
}

impl ShaderLoader {
    pub fn load(device: &Arc<Device>) -> ShaderLoader {
        ShaderLoader {
            vertex_shaders: Self::load_shaders(device, "vertex"),
            fragment_shaders: Self::load_shaders(device, "fragment"),
        }
    }

    pub fn vertex(&self, name: &str) -> Option<&Arc<ShaderModule>> {
        self.vertex_shaders.get(name)
    }

    pub fn fragment(&self, name: &str) -> Option<&Arc<ShaderModule>> {
        self.fragment_shaders.get(name)
    }

    fn load_shaders(device: &Arc<Device>, kind: &str) -> HashMap<String, Arc<ShaderModule>> {
        let mut shaders_path = String::from("./src/shaders/");
        shaders_path.push_str(kind);

        let mut shaders: HashMap<String, Arc<ShaderModule>> = HashMap::new();

        std::fs::read_dir(shaders_path).unwrap().for_each(|e| {
            if let Ok(e) = e {
                if let Ok(_) = e.metadata() {
                    let filename = e.file_name().to_str().unwrap().to_owned();
                    let path = e.path();
                    let ext = path
                        .extension()
                        .expect("Failed to get extension")
                        .to_str()
                        .expect("Failed to convert extension");
                    if ext == "spv" {
                        let path = e.path().to_str().unwrap().to_owned();
                        if let Ok(contents) = std::fs::read(path) {
                            unsafe {
                                if let Ok(module) =
                                    ShaderModule::from_bytes(device.clone(), &contents)
                                {
                                    let name = &filename[..filename.len() - 4];

                                    shaders.insert(name.into(), module);
                                }
                            }
                        }
                    }
                }
            }
        });

        shaders
    }
}
