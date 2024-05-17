use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

fn copy_dir(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            println!("cargo:rerun-if-changed={}", entry.path().to_str().unwrap());
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn main() {
    embuild::espidf::sysenv::output();

    let status = Command::new("pnpm")
        .args(["run", "build"])
        .current_dir("frontend")
        .status()
        .expect("Failed to build frontend project");

    if !status.success() {
        panic!("Frontend build failed");
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    copy_dir(
        Path::new("frontend/dist"),
        Path::new(&out_dir).join("frontend"),
    )
    .expect("Failed to copy frontend dist directory");
}
