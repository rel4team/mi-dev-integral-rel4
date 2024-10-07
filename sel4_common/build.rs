use rust_sel4_pbf_parser::parser::pbf_parser;
fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dir_path = std::path::Path::new(&out_dir).join("pbf");
    match std::fs::create_dir(&dir_path) {
        Ok(_) => println!("Directory created successfully: {}", dir_path.display()),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            println!("Directory already exists: {}", dir_path.display());
        }
        Err(e) => {
            eprintln!("Failed to create directory: {}", e);
        }
    }
    println!("OUT_DIR: {}", out_dir);
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown-target".to_string());
    if target == String::from("aarch64-unknown-none-softfloat") {
        pbf_parser(
            String::from("./pbf/aarch64"),
            dir_path.to_str().unwrap().to_string(),
        );
    } else if target == String::from("riscv64imac-unknown-none-elf") {
        pbf_parser(
            String::from("./pbf/riscv64"),
            dir_path.to_str().unwrap().to_string(),
        );
    }
}
