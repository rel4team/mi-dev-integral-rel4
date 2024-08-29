fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // println!("cargo:rerun-if-changed=data.txt");

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let path = std::path::Path::new(&out_dir).join("test.rs");
    std::fs::write(&path, "pub fn test() { todo!() }").unwrap();
}