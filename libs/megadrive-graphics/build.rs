use std::process::Command;

fn main() {
    Command::new("python").args(&["-m", "pip", "install", "--user", "pipenv"]);
    Command::new("python").args(&["-m", "pipenv", "install"]);
    Command::new("python").args(&["-m", "pipenv", "run", "python", "default_font.py"]);
    println!("cargo:rerun-if-changed=src/default_ascii.rs");
}
