use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/default_ascii.rs");
    Command::new("python")
        .args(&["-m", "pip", "install", "--user", "pipenv"])
        .status()
        .unwrap();
    Command::new("python")
        .args(&["-m", "pipenv", "install"])
        .status()
        .unwrap();
    Command::new("python")
        .args(&["-m", "pipenv", "run", "python", "default_font.py"])
        .status()
        .unwrap();
}
