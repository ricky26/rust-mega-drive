use std::process::Command;

fn main() {
    Command::new("python3")
        .args(&["-m", "pip", "install", "--user", "pipenv"])
        .status()
        .unwrap();
    Command::new("python3")
        .args(&["-m", "pipenv", "install"])
        .status()
        .unwrap();
    Command::new("python3")
        .args(&["-m", "pipenv", "run", "python", "default_font.py"])
        .status()
        .unwrap();

    println!("cargo:rerun-if-changed=default_font.py");
}
