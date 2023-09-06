fn main() {
    use std::env;
    use std::path::Path;
    use std::process::Command;
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("test.exe");
    Command::new("rustc")
        .arg("test_program/test.rs")
        .arg("-o")
        .arg(dest_path)
        .status()
        .unwrap();

    println!("cargo:rerun-if-changed=test_program/test.rs");
}
