use std::{
    fs::{self},
    io,
    path::Path,
};

extern crate shaderc;

fn main() {
    println!("cargo:rerun-if-changed=src/shaders");
    let compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.add_macro_definition("EP", Some("main"));

    let shaders_path = "./src/shaders";

    let compile_shader = |kind: shaderc::ShaderKind, filepath: &str, filename: &str| {
        match fs::read_to_string(filepath) {
            Ok(contents) => {
                match compiler.compile_into_spirv(&contents, kind, filename, "main", Some(&options))
                {
                    Ok(binary) => {
                        if let Some(path) = get_base_path(filepath) {
                            let mut full_path = String::from(path);
                            full_path.push_str(".spv");
                            if let Err(e) = fs::write(full_path, binary.as_binary_u8()) {
                                println!("{}", e.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        println!("COMPILATION ERROR: {}", e.to_string())
                    }
                }
            }
            Err(e) => println!("READ ERROR:{}", e.to_string()),
        };
    };

    let mut process_file = |path: &Path| {
        if let Some(filepath) = path.to_str() {
            if let Some(filename) = path.file_name() {
                if let Some(filename) = filename.to_str() {
                    match path
                        .extension()
                        .expect("Failed to get extension")
                        .to_str()
                        .expect("Failed to convert string")
                    {
                        "vs" => compile_shader(shaderc::ShaderKind::Vertex, filepath, filename),
                        "fs" => compile_shader(shaderc::ShaderKind::Fragment, filepath, filename),
                        _ => {}
                    }
                }
            }
        }
    };

    std::fs::read_dir(shaders_path).unwrap().for_each(|e| {
        if let Ok(e) = e {
            if let Ok(data) = e.metadata() {
                if data.is_dir() {
                    let _ = visit_dirs(e.path().as_path(), &mut process_file);
                } else if data.is_file() {
                    process_file(e.path().as_path());
                }
            }
        }
    });
}

fn get_base_path(filepath: &str) -> Option<&str> {
    if let Some(index) = filepath.rfind(".") {
        Some(&filepath[..index])
    } else {
        None
    }
}

fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&Path)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(entry.path().as_path());
            }
        }
    }
    Ok(())
}
