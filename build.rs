use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=.env");

    let Ok(content) = fs::read_to_string(".env") else {
        return;
    };

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        println!("cargo:rustc-env=BAKED_{}={}", key.trim(), value.trim());
    }
}
