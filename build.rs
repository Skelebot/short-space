extern crate walkdir;

use std::fs::{self, DirBuilder};
use std::path::{Path, PathBuf};
use std::{env, io::Write, process::Command};
use walkdir::WalkDir;

/// A build script that automatically copies the assets/ directiory to the target dir
fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    compile_shaders(&manifest_dir.join("assets"));

    // locate executable path even if the project is in workspace
    let executable_path = locate_target_dir_from_output_dir(&out_dir)
        .expect("failed to find target dir")
        .join(env::var("PROFILE").unwrap());

    copy(
        &manifest_dir.join("assets"),
        &executable_path.join("assets"),
    );
}

fn locate_target_dir_from_output_dir(mut target_dir_search: &Path) -> Option<&Path> {
    loop {
        // if path ends with "target", we assume this is correct dir
        if target_dir_search.ends_with("target") {
            return Some(target_dir_search);
        }

        // otherwise, keep going up in tree until we find "target" dir
        target_dir_search = match target_dir_search.parent() {
            Some(path) => path,
            None => break,
        }
    }

    None
}

/// Compile glsl shaders into SPIR-V
/// Shaders should be in assets/shaders, in format [name].glsl.[extension]
/// Compiled shaders are placed in assets/shaders/compiled, in format [name].[extension].spv
fn compile_shaders(assets_dir: &Path) {
    let shaders_dir = fs::read_dir(assets_dir)
        .unwrap()
        .find(|d| {
            d.as_ref().unwrap().file_type().unwrap().is_dir()
                && &d.as_ref().unwrap().file_name() == "shaders"
        })
        .expect("Couldn't find shaders dir")
        .unwrap()
        .path();

    let shader_files: Vec<PathBuf> = fs::read_dir(&shaders_dir)
        .unwrap()
        // Take files only
        .filter(|f| f.as_ref().unwrap().file_type().unwrap().is_file())
        // Take file paths
        .map(|f| f.unwrap().path())
        // Take only glsl shaders
        .filter(|p| p.file_stem().unwrap().to_str().unwrap().ends_with(".glsl"))
        .collect();

    let compiled_dir = shaders_dir.join("compiled");

    if !compiled_dir.exists() {
        fs::create_dir(&compiled_dir).unwrap()
    }

    for shader in shader_files {
        println!("Compiling shader: {:?}", &shader);

        let mut out_name = {
            let name = shader
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .strip_suffix(".glsl")
                .unwrap();
            compiled_dir.join(name)
        };

        let extension = {
            let mut ext = shader.extension().unwrap().to_owned();
            ext.push(".spv");
            ext
        };

        out_name.set_extension(extension);

        let output = Command::new("glslc")
            .arg(&shader)
            .arg("-o")
            .arg(out_name)
            .output()
            .expect("Failed to execute command");

        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        if !output.status.success() {
            panic!("glslc exited with status code: {:?}", output.status.code())
        }
    }
}

fn copy(from: &Path, to: &Path) {
    let from_path: PathBuf = from.into();
    let to_path: PathBuf = to.into();
    for entry in WalkDir::new(from_path.clone()) {
        let entry = entry.unwrap();

        if let Ok(rel_path) = entry.path().strip_prefix(&from_path) {
            let target_path = to_path.join(rel_path);

            if entry.file_type().is_dir() {
                DirBuilder::new()
                    .recursive(true)
                    .create(target_path)
                    .expect("failed to create target dir");
            } else {
                fs::copy(entry.path(), &target_path).expect("failed to copy");
            }
        }
    }
}
